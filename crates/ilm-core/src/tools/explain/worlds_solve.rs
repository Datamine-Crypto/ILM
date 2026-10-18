use patterns::{because, source};

pub struct PossibleWorlds;
source!(PossibleWorlds, "the logic of public announcements (Plaza; van Ditmarsch, van der Hoek and Kooi): a world says for each person whether a fact of that person holds (a muddy forehead, a red card); a person cannot tell apart two worlds that agree on every fact that the person sees, and knows a statement when it holds in every world that the person cannot tell apart from the world at hand; an announcement keeps the worlds where it holds; a statement follows when it holds in every world that is left. A statement is worked out once for all worlds at a time, as the set of the worlds where it holds, the way a model checker does it");

pub(crate) const MOST_PERSONS: usize = 6;
because!(MOST_PERSONS, PossibleWorlds, "the most persons of a story that the tool reads: the worlds are two to the power of the persons, and a set of worlds is the bits of one number");

pub(crate) type World = u32;
because!(World, PossibleWorlds, "a world as the bits of one number, a bit for the fact of each person");

type WorldSet = u64;
because!(WorldSet, PossibleWorlds, "a set of worlds as the bits of one number, a bit for each world");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Quantity {
    Some,
    Every,
    No,
    NotEvery,
}
because!(Quantity, PossibleWorlds, "a statement about all persons at once: the fact holds for some person, for every person, for no person, or not for every person");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Knowing {
    That,
    Whether,
    NotThat,
    NotWhether,
}
because!(Knowing, PossibleWorlds, "what a person knows of a statement: that it holds, whether it holds, or the negation of either");

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WorldStatement {
    All(Quantity),
    Fact(usize, bool),
    Knows(Knowing, usize, Box<WorldStatement>),
}
because!(WorldStatement, PossibleWorlds, "a statement: about all persons, the fact of one person (or its negation), or what a person knows of a statement");

pub(crate) struct Worlds {
    left: WorldSet,
    sees: Vec<World>,
    persons: usize,
}
because!(Worlds, PossibleWorlds, "the worlds that are left, what each person sees as the bits of the persons whose facts the person sees, and the number of persons");

impl Worlds {
    pub(crate) fn new(persons: usize, sees: Vec<World>) -> Worlds {
        let count = (1 as World) << persons;
        let left = if count as usize >= WorldSet::BITS as usize { WorldSet::MAX } else { ((1 as WorldSet) << count) - 1 };
        Worlds { left, sees, persons }
    }

    fn each(&self) -> impl Iterator<Item = World> + '_ {
        (0..(1 as World) << self.persons).filter(|world| self.left >> world & 1 == 1)
    }

    fn holding(&self, statement: &WorldStatement) -> WorldSet {
        let every: World = ((1 as World) << self.persons) - 1;
        match statement {
            WorldStatement::All(quantity) => self.each().filter(|world| match quantity {
                Quantity::Some => *world != 0,
                Quantity::Every => *world == every,
                Quantity::No => *world == 0,
                Quantity::NotEvery => *world != every,
            }).fold(0, |set, world| set | (1 as WorldSet) << world),
            WorldStatement::Fact(person, positive) => self.each().filter(|world| (world >> person & 1 == 1) == *positive).fold(0, |set, world| set | (1 as WorldSet) << world),
            WorldStatement::Knows(knowing, person, inner) => {
                let holds = self.holding(inner);
                let seen = self.sees[*person];
                let mut classes: Vec<(World, WorldSet)> = Vec::new();
                for world in self.each() {
                    let key = world & seen;
                    match classes.iter_mut().find(|(held, _)| *held == key) {
                        Some((_, set)) => *set |= (1 as WorldSet) << world,
                        None => classes.push((key, (1 as WorldSet) << world)),
                    }
                }
                let mut known: WorldSet = 0;
                for (_, class) in classes {
                    let sure = match knowing {
                        Knowing::That | Knowing::NotThat => class & holds == class,
                        Knowing::Whether | Knowing::NotWhether => class & holds == class || class & holds == 0,
                    };
                    if sure {
                        known |= class;
                    }
                }
                if matches!(knowing, Knowing::That | Knowing::Whether) { known } else { self.left & !known }
            }
        }
    }

    pub(crate) fn announce(&mut self, statement: &WorldStatement) {
        self.left &= self.holding(statement);
    }

    pub(crate) fn follows(&self, statement: &WorldStatement) -> bool {
        self.holding(statement) & self.left == self.left
    }
}
