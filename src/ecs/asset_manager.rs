use crate::rendering::mesh::Mesh;
use std::collections::HashMap;

pub type AssetID = u64;

/// maps asset IDs to mesh data
pub struct AssetManager {
    register: HashMap<AssetID, Mesh>,
    next_id: AssetID,
}

impl AssetManager {
    /// creates a new asset manager
    pub fn new() -> Self {
        Self {
            register: HashMap::new(),
            next_id: 0,
        }
    }

    /// adds asset data to the register
    pub fn add_asset(&mut self, mesh: Mesh) -> AssetID {
        self.register.insert(self.next_id, mesh);
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// makes mesh data available for a given asset id
    pub fn get_asset(&mut self, id: AssetID) -> &Mesh {
        self.register.get(&id).unwrap()
    }
}
