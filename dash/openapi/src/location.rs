#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Geolocation {
    pub latitude: f64,
    pub longitude: f64,
}
