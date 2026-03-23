#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct Girl {
    pub hair_color: HairColor,
    pub skin_color: SkinColor,
    pub body_type: BodyType,
    pub appearance: Appearance,
    pub every_morning: Vec<GirlActions>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum HairColor {
    Black,
    Brown,
    Blonde,
    Red,
}

impl Default for HairColor {
    fn default() -> Self {
        HairColor::Brown
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum SkinColor {
    Yellow,
    Light,
    Dark,
}

impl Default for SkinColor {
    fn default() -> Self {
        SkinColor::Light
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum BodyType {
    Slim,
    Average,
    Curvy,
}

impl Default for BodyType {
    fn default() -> Self {
        BodyType::Average
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Appearance {
    Beautiful,
    Cute,
    Plain,
}

impl Default for Appearance {
    fn default() -> Self {
        Appearance::Cute
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum GirlActions {
    SayHi,
    PrepareBreakfast,
}
