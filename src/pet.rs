use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct PetTag {
    id: u32,
    name: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum PetStatus {
    Available,
    Pending,
    Sold,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum PetCategory {
    Amphibians,
    Birds,
    Insects,
    Reptiles,
    Rodents,
    Canine,
    Feline,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum PetSize {
    Flat,
    House,
    Terraium,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Pet {
    id: u32,
    category: PetCategory,
    photo_urls: String,
    tags: PetTag,
    status: PetStatus,
}
