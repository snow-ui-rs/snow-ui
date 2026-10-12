use crate::elements::Element;
use crate::types::WidgetId;

#[derive(Debug, Clone)]
pub struct Girl {
    pub id: WidgetId,
    pub hair_color: HairColor,
    pub skin_color: SkinColor,
    pub body_type: BodyType,
    pub appearance: Appearance,
    pub every_morning: Vec<GirlActions>,
}

impl Default for Girl {
    fn default() -> Self {
        Self {
            id: WidgetId::new(),
            hair_color: HairColor::default(),
            skin_color: SkinColor::default(),
            body_type: BodyType::default(),
            appearance: Appearance::default(),
            every_morning: Vec::new(),
        }
    }
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
