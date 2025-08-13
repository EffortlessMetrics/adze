use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ParseTableData {
    pub version: u32,            // 1
    pub ts_language_version: u32,// 15
    pub symbol_count: u32,
    pub state_count: u32,
    pub token_count: u32,
    pub external_token_count: u32,
    pub eof_symbol: u16,         // 0
    pub start_symbol: u16,
    pub symbol_names: Vec<String>,

    pub rules: Vec<Rule>,        // stable RuleId == index
    // Sparse maps for compact JSON; use Vec for deterministic order.
    pub actions: Vec<ActionCell>,
    pub gotos: Vec<GotoCell>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rule {
    pub lhs: u16,
    pub rhs_len: u16,
    pub production_id: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionCell {
    pub state: u32,
    pub terminal: u16,        // < token_count + external_token_count
    pub seq: Vec<Action>,     // 1..N
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "k")]
pub enum Action {
    // keep minimal for runtime, record all metadata needed later
    #[serde(rename = "S")] 
    Shift { 
        state: u16, 
        extra: bool, 
        rep: bool 
    },
    #[serde(rename = "R")] 
    Reduce { 
        rule: u16,  // index into `rules`
        dyn_prec: i16, 
        prod: u16 
    },
    #[serde(rename = "A")] 
    Accept,
    #[serde(rename = "E")] 
    Recover,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GotoCell {
    pub state: u32,
    pub nonterminal: u16,   // >= token_count + external_token_count
    pub next_state: u32,    // 0 => none (not emitted)
}