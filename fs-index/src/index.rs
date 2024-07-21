use std::{
    collections::{HashMap, HashSet},
    fs,
    hash::Hash,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};

use data_error::Result;
use data_resource::ResourceId;
use fs_storage::{ARK_FOLDER, INDEX_PATH};

use crate::utils::{discover_paths, scan_entries};

/// The threshold for considering a resource updated
pub const RESOURCE_UPDATED_THRESHOLD: Duration = Duration::from_millis(1);

/// Represents a resource in the index
#[derive(
    PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug, Serialize, Deserialize,
)]
pub struct IndexedResource<Id> {
    /// The unique identifier of the resource
    id: Id,
    /// The path of the resource, relative to the root path
    path: PathBuf,
    /// The last modified time of the resource (from the file system metadata)
    last_modified: SystemTime,
}

impl<Id> IndexedResource<Id> {
    /// Create a new indexed resource
    pub fn new(id: Id, path: PathBuf, last_modified: SystemTime) -> Self {
        IndexedResource {
            id,
            path,
            last_modified,
        }
    }

    /// Return the ID of the resource
    pub fn id(&self) -> &Id {
        &self.id
    }

    /// Return the path of the resource
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Return the last modified time of the resource
    pub fn last_modified(&self) -> SystemTime {
        self.last_modified
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Hash, Clone, Debug)]
pub struct IndexEntry<Id> {
    /// The unique identifier of the resource
    pub(crate) id: Id,
    /// The last modified time of the resource (from the file system metadata)
    pub(crate) last_modified: SystemTime,
}

/// Represents the index of resources in a directory.
///
/// [`ResourceIndex`] provides functionality for managing a directory index,
/// including tracking changes, and querying resources.
///
/// #### Reactive API
/// - [`ResourceIndex::update_all`]: Method to update the index by rescanning
///   files and returning changes (additions/deletions/updates).
///
/// #### Snapshot API
/// - [`ResourceIndex::get_resources_by_id`]: Query resources from the index by
///   ID.
/// - [`ResourceIndex::get_resource_by_path`]: Query a resource from the index
///   by its path.
///
/// #### Track API
/// Allows for fine-grained control over tracking changes in the index
/// - [`ResourceIndex::track_addition`]: Track a newly added file (checks if the
///   file exists in the file system).
/// - [`ResourceIndex::track_removal`]: Track the deletion of a file (checks if
///   the file was actually deleted).
/// - [`ResourceIndex::track_modification`]: Track an update on a single file.
///
/// ## Examples
/// ```no_run
/// use std::path::Path;
/// use fs_index::{ResourceIndex, load_or_build_index};
/// use dev_hash::Crc32;
///
/// // Define the root path
/// let root_path = Path::new("animals");
///
/// // Build the index
/// let index: ResourceIndex<Crc32> = ResourceIndex::build(root_path).expect("Failed to build index");
/// // Store the index
/// index.store().expect("Failed to store index");
///
/// // Load the stored index
/// let mut loaded_index: ResourceIndex<Crc32> = load_or_build_index(root_path, false).expect("Failed to load index");
///
/// // Update the index
/// loaded_index.update_all().expect("Failed to update index");
///
/// // Get a resource by path
/// let _resource = loaded_index
///     .get_resource_by_path("cat.txt")
///     .expect("Resource not found");
/// ```
#[derive(Clone, Debug)]
pub struct ResourceIndex<Id>
where
    Id: Eq + Hash,
{
    /// The root path of the index (canonicalized)
    pub(crate) root: PathBuf,
    /// A map from resource IDs to paths
    ///
    /// Multiple resources can have the same ID (e.g., due to hash collisions
    /// or files with the same content)
    pub(crate) id_to_paths: HashMap<Id, HashSet<PathBuf>>,
    /// A map from resource paths to resources
    pub(crate) path_to_resource: HashMap<PathBuf, IndexEntry<Id>>,
}

/// Represents the result of an update operation on the ResourceIndex
#[derive(PartialEq, Debug)]
pub struct IndexUpdate<Id: ResourceId> {
    /// Resources that were added during the update
    added: HashMap<Id, IndexedResource<Id>>,
    /// Resources that were removed during the update
    removed: HashSet<Id>,
}

impl<Id: ResourceId> IndexUpdate<Id> {
    /// Return the resources that were added during the update
    pub fn added(&self) -> &HashMap<Id, IndexedResource<Id>> {
        &self.added
    }

    /// Return the resources that were removed during the update
    pub fn removed(&self) -> &HashSet<Id> {
        &self.removed
    }
}

impl<Id: ResourceId> ResourceIndex<Id> {
    /// Return the number of resources in the index
    pub fn len(&self) -> usize {
        self.path_to_resource.len()
    }

    /// Return true if the index is empty
    pub fn is_empty(&self) -> bool {
        self.path_to_resource.is_empty()
    }

    /// Return the root path of the index
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Return the resources in the index
    pub fn resources(&self) -> Vec<IndexedResource<Id>> {
        // Using path_to_resource so to avoid not collecting duplicates
        let mut resources = vec![];
        for (path, resource) in self.path_to_resource.iter() {
            resources.push(IndexedResource::new(
                resource.id.clone(),
                path.clone(),
                resource.last_modified,
            ));
        }
        resources
    }

    /// Return the ID collisions
    ///
    /// **Note**: If you are using a cryptographic hash function, collisions
    /// should be files with the same content. If you are using a
    /// non-cryptographic hash function, collisions can be files with the
    /// same content or files whose content hash to the same value.
    pub fn collisions(&self) -> HashMap<Id, HashSet<PathBuf>> {
        // Filter out IDs with only one resource
        self.id_to_paths
            .iter()
            .filter(|(_id, paths)| paths.len() > 1)
            .map(|(id, paths)| (id.clone(), paths.clone()))
            .collect()
    }

    /// Return the number of ID collisions
    ///
    /// **Note**: If you are using a cryptographic hash function, collisions
    /// should be files with the same content. If you are using a
    /// non-cryptographic hash function, collisions can be files with the
    /// same content or files whose content hash to the same value.
    pub fn num_collisions(&self) -> usize {
        // Aggregate the number of collisions for each ID
        self.id_to_paths
            .values()
            .filter(|paths| paths.len() > 1)
            .map(|paths| paths.len())
            .sum()
    }

    /// Save the index to the file system (as a JSON file in
    /// <root_path>/ARK_FOLDER/INDEX_PATH)
    pub fn store(&self) -> Result<()> {
        let ark_folder = self.root.join(ARK_FOLDER);
        let index_path = ark_folder.join(INDEX_PATH);
        log::debug!("Storing index at: {:?}", index_path);

        fs::create_dir_all(&ark_folder)?;
        let index_file = fs::File::create(index_path)?;
        serde_json::to_writer_pretty(index_file, self)?;

        Ok(())
    }

    /// Get resources by their ID
    ///
    /// Returns None if there is no resource with the given ID
    ///
    /// **Note**: This can return multiple resources with the same ID in case of
    /// hash collisions or files with the same content
    pub fn get_resources_by_id(
        &self,
        id: &Id,
    ) -> Option<Vec<IndexedResource<Id>>> {
        let mut resources = vec![];

        let paths = self.id_to_paths.get(id)?;
        for path in paths {
            let resource = self.path_to_resource.get(path)?;
            let resource = IndexedResource::new(
                resource.id.clone(),
                path.clone(),
                resource.last_modified,
            );
            resources.push(resource);
        }

        Some(resources)
    }

    /// Get a resource by its path
    ///
    /// Returns None if the resource does not exist
    ///
    /// **Note**: The path should be relative to the root path
    pub fn get_resource_by_path<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Option<IndexedResource<Id>> {
        let resource = self.path_to_resource.get(path.as_ref())?;
        let resource = IndexedResource::new(
            resource.id.clone(),
            path.as_ref().to_path_buf(),
            resource.last_modified,
        );
        Some(resource)
    }

    /// Build a new index from the given root path
    pub fn build<P: AsRef<Path>>(root_path: P) -> Result<Self> {
        log::debug!("Building index at root path: {:?}", root_path.as_ref());

        // Canonicalize the root path
        let root = fs::canonicalize(&root_path)?;
        let mut id_to_paths: HashMap<Id, HashSet<PathBuf>> = HashMap::new();
        let mut path_to_resource = HashMap::new();

        // Discover paths in the root directory
        let paths = discover_paths(&root)?;
        let entries: HashMap<PathBuf, IndexedResource<Id>> =
            scan_entries(paths);

        // Strip the root path from the entries
        let entries: HashMap<PathBuf, IndexEntry<Id>> = entries
            .into_iter()
            .map(|(path, resource)| {
                let relative_path =
                    path.strip_prefix(&root).unwrap().to_path_buf();
                let resource = IndexEntry {
                    id: resource.id().clone(),
                    last_modified: resource.last_modified(),
                };

                // Update the ID to paths map
                id_to_paths
                    .entry(resource.id.clone())
                    .or_default()
                    .insert(relative_path.clone());

                (relative_path, resource)
            })
            .collect();

        // Update the path to resource map
        path_to_resource.extend(entries.clone());

        let index = ResourceIndex {
            root,
            id_to_paths,
            path_to_resource,
        };
        Ok(index)
    }

    /// Update the index with the latest information from the file system
    pub fn update_all(&mut self) -> Result<IndexUpdate<Id>> {
        log::debug!("Updating index at root path: {:?}", self.root);
        log::trace!("Current index: {:#?}", self);

        let mut added: HashMap<Id, IndexedResource<Id>> = HashMap::new();
        let mut removed: HashSet<Id> = HashSet::new();

        let current_paths = discover_paths(&self.root)?;

        // Assuming that collection manipulation is faster than repeated
        // lookups
        let current_entries: HashMap<PathBuf, IndexedResource<Id>> =
            scan_entries(current_paths.clone());
        let previous_entries = self.path_to_resource.clone();
        // `preserved_entries` is the intersection of current_entries and
        // previous_entries
        let preserved_entries: HashMap<PathBuf, IndexEntry<Id>> =
            current_entries
                .iter()
                .filter_map(|(path, _resource)| {
                    previous_entries.get(path).map(|prev_resource| {
                        (path.clone(), prev_resource.clone())
                    })
                })
                .collect();

        // `created_entries` is the difference between current_entries and
        // preserved_entries
        let created_entries: HashMap<PathBuf, IndexedResource<Id>> =
            current_entries
                .iter()
                .filter_map(|(path, resource)| {
                    if preserved_entries.contains_key(path) {
                        None
                    } else {
                        Some((path.clone(), resource.clone()))
                    }
                })
                .collect();

        // `updated_entries` is the intersection of current_entries and
        // preserved_entries where the last modified time has changed
        // significantly (> RESOURCE_UPDATED_THRESHOLD)
        let updated_entries: HashMap<PathBuf, IndexedResource<Id>> =
            current_entries
                .into_iter()
                .filter(|(path, entry)| {
                    if !preserved_entries.contains_key(path) {
                        false
                    } else {
                        let our_entry = &self.path_to_resource[path];
                        let prev_modified = our_entry.last_modified;

                        let result = entry.path().metadata();
                        match result {
                            Err(msg) => {
                                log::error!(
                                    "Couldn't retrieve metadata for {}: {}",
                                    &path.display(),
                                    msg
                                );
                                false
                            }
                            Ok(metadata) => match metadata.modified() {
                                Err(msg) => {
                                    log::error!(
                                    "Couldn't retrieve timestamp for {}: {}",
                                    &path.display(),
                                    msg
                                );
                                    false
                                }
                                Ok(curr_modified) => {
                                    let elapsed = curr_modified
                                        .duration_since(prev_modified)
                                        .unwrap();

                                    let was_updated =
                                        elapsed >= RESOURCE_UPDATED_THRESHOLD;
                                    if was_updated {
                                        log::trace!(
                                            "[update] modified {} by path {}
                                        \twas {:?}
                                        \tnow {:?}
                                        \telapsed {:?}",
                                            our_entry.id,
                                            path.display(),
                                            prev_modified,
                                            curr_modified,
                                            elapsed
                                        );
                                    }

                                    was_updated
                                }
                            },
                        }
                    }
                })
                .collect();

        // Remove resources that are not in the current entries
        let removed_entries: HashMap<PathBuf, IndexEntry<Id>> =
            previous_entries
                .iter()
                .filter_map(|(path, resource)| {
                    if preserved_entries.contains_key(path) {
                        None
                    } else {
                        Some((path.clone(), resource.clone()))
                    }
                })
                .collect();
        for (path, resource) in removed_entries {
            log::trace!(
                "Resource removed: {:?}, last modified: {:?}",
                path,
                resource.last_modified
            );

            self.path_to_resource.remove(&path);
            self.id_to_paths
                .get_mut(&resource.id)
                .unwrap()
                .remove(&path);
            let id = resource.id.clone();
            // Only remove the ID if it has no paths
            if self.id_to_paths[&id].is_empty() {
                self.id_to_paths.remove(&id);
                removed.insert(id);
            }
        }

        let added_entries: HashMap<PathBuf, IndexedResource<Id>> =
            updated_entries
                .iter()
                .chain(created_entries.iter())
                .filter_map(|(path, resource)| {
                    if self.path_to_resource.contains_key(path) {
                        None
                    } else {
                        Some((path.clone(), resource.clone()))
                    }
                })
                .collect();

        for (path, resource) in added_entries {
            log::trace!("Resource added: {:?}", path);

            // strip the root path from the path
            let relative_path = path
                .strip_prefix(&self.root)
                .unwrap()
                .to_path_buf();
            let resource = IndexedResource::new(
                resource.id().clone(),
                relative_path.clone(),
                resource.last_modified(),
            );
            let index_entry_resource = IndexEntry {
                id: resource.id().clone(),
                last_modified: resource.last_modified(),
            };

            self.path_to_resource
                .insert(relative_path.clone(), index_entry_resource.clone());
            let id = resource.id.clone();
            self.id_to_paths
                .entry(id.clone())
                .or_default()
                .insert(relative_path.clone());
            added.insert(id, resource);
        }

        Ok(IndexUpdate { added, removed })
    }
}
