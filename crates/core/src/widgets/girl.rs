use crate::elements::Element;

#[derive(Debug, Default, Clone)]
pub struct Girl {
    pub hair_color: HairColor,
    pub skin_color: SkinColor,
    pub body_type: BodyType,
    pub appearance: Appearance,
    pub every_morning: Vec<GirlActions>,
}

#[derive(Debug, Clone, Copy)]
pub enum HairColor {
    Black,
    Brown,
    Blonde,
    Red,
}

impl Default for HairColor {
    fn default() -> Self {
        Self::Brown
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SkinColor {
    Yellow,
    Light,
    Dark,
}

impl Default for SkinColor {
    fn default() -> Self {
        Self::Light
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BodyType {
    Slim,
    Average,
    Curvy,
}

impl Default for BodyType {
    fn default() -> Self {
        Self::Average
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Appearance {
    Beautiful,
    Cute,
    Plain,
}

impl Default for Appearance {
    fn default() -> Self {
        Self::Cute
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GirlActions {
    SayHi,
    PrepareBreakfast,
}

impl From<Girl> for Element {
    fn from(girl: Girl) -> Self {
        Element::Girl(girl)
    }
}
