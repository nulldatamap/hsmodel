extern crate rand;

use std::sync::Arc;

use rand::{thread_rng, Rng};

use PlayerIndex::*;

type Action = fn( Game ) -> Game;
type CardId        = usize;
type ProtoHeroId   = usize;
type ProtoMinionId = usize;
type ProtoSpellId  = usize;
type ProtoWeaponId = usize;
type CharacterId   = u32;

type Placeholder = usize;


struct Content {
  cards   : Vec<ProtoCard>,
  heroes  : Vec<ProtoHero>,
  minions : Vec<ProtoMinion>,
  spells  : Vec<ProtoSpell>,
  weapons : Vec<ProtoWeapon>
}

#[derive(Clone)]
struct Game {
  turn     : Turn,
  next_id  : CharacterId,
  player1  : Player,
  player2  : Player,
  board    : Board,
  rules    : Vec<Rule>,
  triggers : Vec<Trigger>,
  content  : Arc<Content>
}

impl Game {
  fn new( content : Arc<Content>
        , h1      : ProtoHeroId, d1 : Vec<CardId>
        , h2      : ProtoHeroId, d2 : Vec<CardId> ) -> Game {

    let hero1 = Hero::from_proto( Character::raw( 0 ), &content.heroes[h1] );
    let hero2 = Hero::from_proto( Character::raw( 1 ), &content.heroes[h2] );

    Game { turn    : Turn::intial_turn()
         , next_id : 2 // Id 0 and 1 are used for the two heroes
         , player1 : Player::new( Player1, d1 )
         , player2 : Player::new( Player2, d2 )
         , board   : Board::new( hero1, hero2 )
         , rules   : Vec::new()
         , triggers: Vec::new()
         , content : content }
  }

  fn step( mut self ) -> Game {
    use TurnState::*;

    match self.turn.state {
      Mulligan => panic!( "wut" ),
      GameStart => {
        self = self.shuffle_deck( Player1 ).draw( Player1, 3 )
                   .shuffle_deck( Player2 ).draw( Player2, 3 );
        
        self.turn = self.turn.successor();
      },
      TurnStart => {
        // TODO: Trigger turn start
        let p = self.turn.current_player;
        self = self.draw( p, 1 );

        // TODO: Make this more generic
        {
          let player = self.current_player();
          if player.base_mana < 10 {
            player.base_mana += 1;
          }
          player.mana = player.base_mana;
        }

        self.turn = self.turn.successor();
      },
      TurnPlay => {
        // TODO: Get user action
        self.turn = self.turn.successor();
      },
      TurnEnd => {
        // TODO: Trigger turn end
        self.turn = self.turn.successor();
      },
      GameEnd => {
        println!( "Game over!" );
      }
    }

    self
  }

  fn player( &mut self, p : PlayerIndex ) -> &mut Player {
    match p {
      Player1 => &mut self.player1,
      Player2 => &mut self.player2
    }
  }

  fn current_player( &mut self ) -> &mut Player {
    let p = self.turn.current_player;
    self.player( p )
  }

  fn enemy_player( &mut self ) -> &mut Player {
    let p = self.turn.current_player.other();
    self.player( p )
  }

  fn shuffle_deck( mut self, p : PlayerIndex ) -> Game {
    thread_rng().shuffle( &mut self.player( p ).deck[..] );

    self
  }

  fn draw( mut self, p : PlayerIndex, count : usize ) -> Game {
    {
      let content = self.content.clone();
      let player = self.player( p );

      for i in 0..count {
        match player.deck.pop() {
          Some( c ) => {
            // TOOD: Trigger card drawn
            player.hand.push( Card::from_proto( &content.cards[c] ) );
          },
          None => {
            // Trigger empty draw
          }
        }
      }
    }

    self
  }

}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Turn {
  turn_count      : u32,
  starting_player : PlayerIndex,
  current_player  : PlayerIndex,
  state           : TurnState
}

impl Turn {
  fn intial_turn() -> Turn {
    Turn { turn_count     : 0
         , starting_player: Player1
         , current_player : Player1
         , state          : TurnState::GameStart } // TODO: Add mulligan
  }

  fn successor( mut self ) -> Turn {
    use TurnState::*;

    match self.state {
      Mulligan  => panic!( "unimplemented" ),
      GameStart => {
        self.state = TurnStart;
        self.turn_count += 1;
      },
      TurnStart => self.state = TurnPlay,
      TurnPlay  => self.state = TurnEnd,
      TurnEnd   => {
        self.state = TurnStart;
        self.turn_count += 1;
        self.current_player = self.current_player.other();
      },
      GameEnd => {}
    }

    self
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum TurnState {
  Mulligan,
  GameStart,
  TurnStart,
  TurnPlay,
  TurnEnd,
  GameEnd
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum PlayerIndex {
  Player1,
  Player2
}

impl PlayerIndex {
  fn other( self ) -> PlayerIndex {
    match self {
      Player1 => Player2,
      Player2 => Player1
    }
  }
}

#[derive(Debug, Clone)]
struct Player {
  index     : PlayerIndex,
  hand      : Vec<Card>,
  base_mana : u32,
  mana      : u32,
  effects   : Vec<Effect>,
  deck      : Vec<CardId>,
  graveyard : Vec<CardId>
}

impl Player {
  fn new( i : PlayerIndex, deck : Vec<CardId> ) -> Player {
    Player { index    : i
           , hand     : Vec::new()
           , base_mana: 0
           , mana     : 0
           , effects  : Vec::new()
           , deck     : deck
           , graveyard: Vec::new() }
  }
}

#[derive(Debug, Clone)]
struct Card {
  proto     : CardId,
  cost      : u32,
  kind      : CardKind
}

impl Card {
  fn from_proto( proto : &ProtoCard ) -> Card {
    Card { proto: proto.id
         , cost : proto.cost
         , kind : proto.kind }
  }
}

#[derive(Debug, Clone)]
struct ProtoCard {
  id         : CardId,
  title      : &'static str,
  text       : &'static str,
  lore       : &'static str,
  tribe      : Tribe,
  categories : Vec<Category>,
  cost       : u32,
  rarety     : Rarety,
  class      : Class,
  set        : Set,
  golden     : bool,
  kind       : CardKind
}

#[derive(Debug, Clone, Copy)]
enum CardKind {
  HeroCard( ProtoHeroId ),
  HeroPowerCard( ProtoSpellId ),
  MinionCard( ProtoMinionId ),
  SpellCard( ProtoSpellId ),
  WeaponCard( ProtoWeaponId )
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Rarety {
  Basic,
  Common,
  Rare,
  Epic,
  Legendary
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Class {
  Neutral,
  Warrior,
  Mage,
  Paladin,
  Rogue,
  Hunter,
  Priest,
  Warlock,
  Shaman,
  Druid
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Set {
  Basic
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Tribe {
  Beast,
  Demon,
  Dragon,
  Mech,
  Murloc,
  Pirate,
  Totem,
  General
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Category;

#[derive(Debug, PartialEq, Eq, Clone)]
enum Metadata {
  None
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Effect {
  None
}

#[derive(Clone)]
struct ProtoHero {
  id          : ProtoHeroId,
  base_armor  : u32,
  base_health : i32,
  base_attack : u32,
  hero_power  : ProtoSpellId,
  action      : fn( Game, Hero ) -> ( Game, Hero )
}


struct ProtoMinion {
  id          : ProtoMinionId,
  base_health : i32,
  health      : i32,
  attack      : u32,
  precast     : fn( &Game ) -> (Metadata, bool),
  action      : fn( Game, Minion, Metadata ) -> (Game, Minion)
}

impl Clone for ProtoMinion {
  fn clone( &self ) -> ProtoMinion {
    ProtoMinion { id: self.id
                , base_health: self.base_health
                , health: self.health
                , attack: self.attack
                , precast: self.precast
                , action: self.action }
  }
}


struct ProtoSpell {
  id      : ProtoSpellId,
  precast : fn( &Game ) -> (Metadata, bool),
  action  : fn( Game, Metadata ) -> Game
}

impl Clone for ProtoSpell {
  fn clone( &self ) -> ProtoSpell {
    ProtoSpell { id: self.id
               , precast: self.precast
               , action: self.action }
  }
}


struct ProtoWeapon {
  id         : ProtoWeaponId,
  attack     : u32,
  durability : u32,
  precast    : fn( &Game ) -> (Metadata, bool),
  action     : fn( Game, Weapon, Metadata ) -> (Game, Weapon)
}


impl Clone for ProtoWeapon {
  fn clone( &self ) -> ProtoWeapon {
    ProtoWeapon { id: self.id
                , attack: self.attack
                , durability: self.durability
                , precast: self.precast
                , action: self.action }
  }
}

#[derive(Clone)]
struct Hero {
  proto      : ProtoHeroId,
  character  : Character,
  armor      : u32,
  hero_power : HeroPower,
  weapon     : Option<Weapon>,
}

impl Hero {
  fn from_proto( mut c : Character, proto : &ProtoHero ) -> Hero {
    c.base_health = proto.base_health;
    c.health      = c.base_health;
    c.attack      = proto.base_attack;
    Hero { proto     : proto.id
         , character : c
         , armor     : proto.base_armor
         , hero_power: HeroPower::new( proto.hero_power )
         , weapon    : None }
  }
}

#[derive(Debug, Clone)]
struct Minion {
  proto     : ProtoMinionId,
  character : Character
}

#[derive(Debug, Clone)]
struct Weapon {
  proto      : ProtoWeaponId,
  damage     : u32,
  durability : u32,
  effects    : Vec<Effect>
}

#[derive(Debug, Clone)]
struct Character {
  id            : CharacterId,
  base_health   : i32,
  health        : i32,
  attack        : u32,
  attack_counts : u32,
  effects       : Vec<Effect>
}

impl Character {
  fn raw( id : CharacterId ) -> Character {
    Character { id           : id
              , base_health  : 0
              , health       : 0
              , attack       : 0
              , attack_counts: 0
              , effects      : Vec::new() }
  }
}

#[derive(Debug, Clone)]
struct HeroPower {
  uses    : u32,
  proto   : ProtoSpellId,
  effects : Vec<Effect>
}

impl HeroPower {
  fn new( proto : ProtoSpellId ) -> HeroPower {
    HeroPower { uses   : 0
              , proto  : proto
              , effects: Vec::new() }
  }
}

#[derive(Clone)]
struct Board {
  half1   : Half,
  half2   : Half,
  effects : Vec<Effect>
}

impl Board {
  fn new( hero1 : Hero, hero2 : Hero ) -> Board {
    Board { half1  : Half::new( hero1 )
          , half2  : Half::new( hero2 )
          , effects: Vec::new() }
  }
}

#[derive(Clone)]
struct Half {
  hero    : Hero,
  minions : Vec<Minion>,
  effects : Vec<Effect>
}

impl Half {
  fn new( hero : Hero ) -> Half {
    Half { hero   : hero
         , minions: Vec::new()
         , effects: Vec::new() }
  }
}

#[derive(Clone)]
struct Rule {
  init : Action,
  end  : Action
}

impl PartialEq<Rule> for Rule {
  fn eq( &self, other : &Rule ) -> bool {
    self.init as usize == other.init as usize
    && self.end as usize == other.end as usize
  }
}

impl Eq for Rule {}


#[derive(Clone)]
struct Trigger {
  trigger_events : Vec<TriggerEvent>,
  action         : Action
}

impl PartialEq<Trigger> for Trigger {
  fn eq( &self, other : &Trigger ) -> bool {
    self.trigger_events == other.trigger_events
    && self.action as usize == other.action as usize
  }
}

impl Eq for Trigger {}

#[derive(Debug, PartialEq, Eq, Clone)]
enum TriggerEvent {
  TurnStard,
  TurnEnd,
  CardDrawn,
  CardPlayed,
  SpellCasted,
  MinionSummoned,
  WeaponEquipped,
  MinionSpawned,
  MinionAttacking,
  MinionAttacked,
  HeroAttacking,
  HeroAttacked,
  HeroPowerUsed,
  HeroTargeted,
  MinionTargeted,
}

fn hero_action_nop( g : Game, h : Hero ) -> (Game, Hero) {
  (g, h)
}

fn main() {
  let mage = ProtoHero { id: 0
                       , base_armor: 0
                       , base_health: 30
                       , base_attack: 0
                       , hero_power: 0
                       , action: hero_action_nop };

  let mage_card = ProtoCard { id: 0
                            , title: "Jaina"
                            , text: "Hero"
                            , lore: "Her magic will tare you appart."
                            , tribe: Tribe::General
                            , categories: Vec::new()
                            , cost: 0
                            , rarety: Rarety::Basic
                            , class: Class::Mage
                            , set: Set::Basic
                            , golden: false
                            , kind: CardKind::HeroCard( 0 ) };

  let content = Arc::new( Content { cards: vec![ mage_card ]
                                  , heroes: vec![ mage ]
                                  , minions: Vec::new()
                                  , spells: Vec::new()
                                  , weapons: Vec::new() } );

  let mut game = Game::new( content.clone(), 0, Vec::new(), 0, Vec::new() );
  
  game = game.step();

}
