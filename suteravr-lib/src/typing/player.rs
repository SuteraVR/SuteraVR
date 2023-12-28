use alkahest::alkahest;

#[derive(Debug, PartialEq, Eq)]
#[alkahest(Formula, SerializeRef, Deserialize)]
struct Player {
    displayname: String,
}
