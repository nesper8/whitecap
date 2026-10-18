use crate::archive::path;

pub(crate) fn normalized_entry_path(name: &str) -> String {
    path::normalize(&name.replace('\\', "/"))
}
