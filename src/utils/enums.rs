use crate::impl_Display;
use clap::ValueEnum;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
pub enum VersionGroup {
  RedBlue,
  Yellow,
  GoldSilver,
  Crystal,
  RubySapphire,
  Emerald,
  FireredLeafgreen,
  Colusseum,
  #[value(alias = "xd")]
  XD,
  DiamondPearl,
  Platinum,
  HeartgoldSoulsilver,
  BlackWhite,
  Black2White2,
  XY,
  OmegaRubyAlphaSapphire,
  SunMoon,
  UltraSunUltraMoon,
  LetsGoPikachuLetsGoEevee,
  SwordShield,
  TheIsleOfArmor,
  TheCrownTundra,
  BrilliantDiamondShiningPearl,
  LegendsArceus,
  ScarletViolet,
  TheTealMask,
  TheIndigoDisk,
  #[value(alias = "legends-za")]
  LegendsZA,
  MegaDimension,
}
impl_Display!(VersionGroup);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
pub enum Version {
  Red,
  Blue,
  Yellow,
  Gold,
  Silver,
  Crystal,
  Ruby,
  Sapphire,
  Emerald,
  Firered,
  Leafgreen,
  Diamond,
  Pearl,
  Platinum,
  Heartgold,
  Soulsilver,
  Black,
  White,
  Colosseum,
  #[value(alias = "xd")]
  XD,
  Black2,
  White2,
  X,
  Y,
  OmegaRuby,
  AlphaSapphire,
  Sun,
  Moon,
  UltraSun,
  UltraMoon,
  LetsGoPikachu,
  LetsGoEevee,
  Sword,
  Shield,
  TheIsleOfArmor,
  TheCrownTundra,
  BrilliantDiamond,
  ShiningPearl,
  LegendsArceus,
  Scarlet,
  Violet,
  TheTealMask,
  TheIndigoDisk,
  #[value(alias = "legends-za")]
  LegendsZA,
  MegaDimension,
}
impl_Display!(Version);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
pub enum Type {
  Normal,
  Fighting,
  Flying,
  Poison,
  Ground,
  Rock,
  Bug,
  Ghost,
  Steel,
  Fire,
  Water,
  Grass,
  Electric,
  Psychic,
  Ice,
  Dragon,
  Dark,
  Fairy,
}
impl_Display!(Type);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
pub enum LanguageId {
  #[value(alias = "ja-Hrkt")]
  JaHrkt,
  Roomaji,
  Ko,
  #[value(alias = "zh-Hant")]
  ZhHant,
  Fr,
  De,
  Es,
  It,
  En,
  Cs,
  Ja,
  #[value(alias = "zh-Hans")]
  ZhHans,
  #[value(alias = "pt-BR")]
  PtBR,
}
impl_Display!(LanguageId);
