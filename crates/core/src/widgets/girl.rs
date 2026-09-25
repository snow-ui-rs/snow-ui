use crate::elements::Element;

#[derive(Debug, Default, Clone)]
pub struct Girl {
    pub hair_color: HairColor,
    pub skin_color: SkinColor,
    pub body_type: BodyType,
    pub appearance: Appearance,
    pub every_morning: Vec<GirlActions>,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum HairColor {
    Black,
    #[default]
    Brown,
    Blonde,
    Red,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum SkinColor {
    Yellow,
    #[default]
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum BodyType {
    Slim,
    #[default]
    Average,
    Curvy,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Appearance {
    Beautiful,
    #[default]
    Cute,
    Plain,
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
