pub trait IdGenerator {
    type Error;
    type Id;

    fn next_id(&self) -> std::result::Result<Self::Id, Self::Error>;
}

pub trait NextAvailId {
    fn next_avail_id(&self) -> Option<&std::time::Duration>;
}
