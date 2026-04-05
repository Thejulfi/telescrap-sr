#[derive(Debug, Clone)]
pub enum ClubType {
    StadeRochelais,
    UnionBordeauxBegles, 
}

#[derive(Debug, Clone)]
pub struct Club {
    pub name: String,
    pub club_type: ClubType,
    pub url: String,
}

impl Club {
    pub fn new(name: String, club_type: ClubType, url: String) -> Self {
        Self { name, club_type, url }
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn get_type_from_name(name: &str) -> ClubType {
        match name.to_lowercase().as_str() {
            "staderochelais" => ClubType::StadeRochelais,
            "unionbordeauxbegles" => ClubType::UnionBordeauxBegles,
            _ => panic!("Unknown club name: {}", name),
        }
    }
}