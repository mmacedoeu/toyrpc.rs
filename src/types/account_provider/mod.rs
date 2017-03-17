/// Dapp identifier
#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct DappId(String);

impl From<DappId> for String {
    fn from(id: DappId) -> String {
        id.0
    }
}
impl From<String> for DappId {
    fn from(id: String) -> DappId {
        DappId(id)
    }
}
impl<'a> From<&'a str> for DappId {
    fn from(id: &'a str) -> DappId {
        DappId(id.to_owned())
    }
}
