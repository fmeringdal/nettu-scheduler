pub struct KVMetadata {
    pub key: String,
    pub value: String,
}

pub struct MetadataFindInput {
    pub limit: usize,
    pub skip: usize,
    pub metadata: KVMetadata,
}

impl MetadataFindInput {
    pub(crate) fn to_query_string(&self) -> String {
        format!(
            "skip={}&limit={}&key={}&value={}",
            self.skip, self.limit, self.metadata.key, self.metadata.value
        )
    }
}
