use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Data {
    pub data: Vec<(i32, String, i64)>,
}

impl Default for Data {
    fn default() -> Self {
        let data = load_data_from_str(include_str!("table_data.json")).expect("Failed to load data from embedded JSON");
        Self { data }
    }
}

fn load_data_from_str(json_str: &str) -> Result<Vec<(i32, String, i64)>, Box<dyn std::error::Error>> {
    let json_data: Vec<serde_json::Value> = serde_json::from_str(json_str)?;
    let mut data = Vec::new();
    for item in json_data {
        let id = item["id"].as_i64().unwrap() as i32;
        let name = item["name"].as_str().unwrap().to_string();
        let value = item["value"].as_i64().unwrap();
        data.push((id, name, value));
    }
    Ok(data)
}

pub enum DataActions {
    #[allow(dead_code)]
    RemoveData(i32),
}

impl yew::Reducible for Data {
    type Action = DataActions;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        let mut new = (*self).clone();
        match action {
            DataActions::RemoveData(id) => {
                new.data.retain(|(i, _, _)| i != &id);
            }
        }
        std::rc::Rc::new(new)
    }
}