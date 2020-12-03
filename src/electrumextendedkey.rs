pub trait ElectrumExtendedKey {
    fn to_descriptors(&self) -> Vec<String>;
}
