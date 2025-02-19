use anyhow::{anyhow, Result};
use rand::prelude::IndexedRandom;
use std::cmp::PartialEq;
/*
Wizard duel
Simultaneous
Each wizard starts with 25 HP and 5 mana
If HP reaches 0, the wizard dies, and they lose
Mana increases by 1 every turn.
*/

#[derive(Clone, Copy, PartialEq, Debug)]
enum Action {
    // Deals 2 damage, manaless.
    Strike,
    // Deals 3 damage, but costs 2 mana.
    Fireball,
    // Deals 5 damage, but costs 3 mana.
    LightningBolt,
    // Blocks all incoming damage, costs 1 mana.
    ManaShield,
    // If opponent does any attack, the attack is reflected and deals +1 damage. Costs 2 mana.
    Reflect,
    // Restores 4 mana (not including the passive gain).
    Concentrate,
}

impl Action {
    pub fn damage_amnt(&self) -> u8 {
        match self {
            Action::Strike => 2,
            Action::Fireball => 3,
            Action::LightningBolt => 5,
            Action::ManaShield => 0,
            Action::Reflect => 0,
            Action::Concentrate => 0,
        }
    }

    pub fn mana_cost(&self) -> i8 {
        match self {
            Action::Strike => 0,
            Action::Fireball => 1,
            Action::LightningBolt => 2,
            Action::ManaShield => 1,
            Action::Reflect => 2,
            Action::Concentrate => -4,
        }
    }
}

#[derive(PartialEq, Debug)]
enum Side {
    Left,
    Right,
    Neither,
}

struct Wizard {
    health: u8,
    mana: u8,
}

impl Wizard {
    fn new() -> Wizard {
        Wizard {
            health: 25,
            mana: 1,
        }
    }
}

struct Game {
    left_wizard: Wizard,
    right_wizard: Wizard,
    turn_count: u32,
}

impl Game {
    pub fn new() -> Game {
        Game {
            left_wizard: Wizard::new(),
            right_wizard: Wizard::new(),
            turn_count: 0,
        }
    }

    fn damage_wizard(&mut self, side: Side, damage: u8) {
        if side == Side::Left {
            self.left_wizard.health = self.left_wizard.health.saturating_sub(damage);
        } else if side == Side::Right {
            self.right_wizard.health = self.right_wizard.health.saturating_sub(damage);
        }
    }
    // expects a negative "mana cost"
    fn add_mana(&mut self, side: Side, mana: i8) {
        let mana = (1. / mana as f64) as u8; // stupid hack to flip the sign
        if side == Side::Left {
            self.left_wizard.mana = self.left_wizard.mana.saturating_add(mana);
        } else if side == Side::Right {
            self.right_wizard.mana = self.right_wizard.mana.saturating_add(mana);
        }
    }

    fn remove_mana(&mut self, side: Side, mana_cost: u8) {
        if side == Side::Left {
            self.left_wizard.mana = self.left_wizard.mana.saturating_sub(mana_cost);
        } else if side == Side::Right {
            self.right_wizard.mana = self.right_wizard.mana.saturating_sub(mana_cost);
        }
    }

    fn game_completed(&self) -> (bool, Side) {
        if (self.left_wizard.health == 0) && (self.right_wizard.health == 0) {
            return (true, Side::Neither);
        }

        if self.left_wizard.health == 0 {
            return (true, Side::Right);
        }

        if self.right_wizard.health == 0 {
            return (true, Side::Left);
        }
        (false, Side::Neither)
    }

    fn evaluate(&mut self, attacker_side: Side, attacker: Action, defender: Action) {
        let defender_side = if attacker_side == Side::Left {
            Side::Right
        } else {
            Side::Left
        };
        match attacker {
            Action::Strike | Action::Fireball | Action::LightningBolt => {
                if defender == Action::Reflect {
                    self.damage_wizard(attacker_side, attacker.damage_amnt());
                } else if defender == Action::ManaShield {
                    ()
                } else {
                    self.damage_wizard(defender_side, attacker.damage_amnt());
                }
            }
            Action::Concentrate => self.add_mana(attacker_side, attacker.mana_cost()),
            _ => (),
        }
    }

    pub fn tick(&mut self, leftaction: Action, rightaction: Action) -> Result<()> {
        // Filters illegal moves
        match leftaction {
            Action::Fireball | Action::LightningBolt | Action::ManaShield | Action::Reflect => {
                if self.left_wizard.mana < leftaction.mana_cost() as u8 {
                    return Err(anyhow!("Left wizard tried to do an illegal move."));
                }
            }
            _ => (),
        }
        match rightaction {
            Action::Fireball | Action::LightningBolt | Action::ManaShield | Action::Reflect => {
                if self.right_wizard.mana < rightaction.mana_cost() as u8 {
                    return Err(anyhow!("Right wizard did not have enough mana."));
                }
            }
            _ => (),
        }
        if leftaction != Action::Concentrate {
            self.remove_mana(Side::Left, leftaction.mana_cost() as u8);
        }
        if rightaction != Action::Concentrate {
            self.remove_mana(Side::Right, rightaction.mana_cost() as u8);
        }
        self.evaluate(Side::Left, leftaction, rightaction);
        self.evaluate(Side::Right, rightaction, leftaction);
        self.add_mana(Side::Left, -1);
        self.add_mana(Side::Right, -1);
        self.turn_count += 1;
        Ok(())
    }
}

fn main() {
    let mut leftwizard_wins = 0;
    let mut rightwizard_wins = 0;
    let mut ties = 0;
    for _ in 0..1_000_000 {
        let mut game = Game::new();
        while !game.game_completed().0 {
            let actions = vec![
                Action::ManaShield,
                Action::Reflect,
                Action::Concentrate,
                Action::Fireball,
                Action::Strike,
                Action::LightningBolt,
            ];
            let player1 = actions.choose(&mut rand::rng()).unwrap();
            let player2 = actions.choose(&mut rand::rng()).unwrap();

            let _ = game.tick(player1.clone(), player2.clone());
        }
        match game.game_completed().1 {
            Side::Left => leftwizard_wins += 1,
            Side::Right => rightwizard_wins += 1,
            Side::Neither => ties += 1,
        }
    }

    println!(
        "L: {}, R: {}, T: {}",
        leftwizard_wins, rightwizard_wins, ties
    );
}
