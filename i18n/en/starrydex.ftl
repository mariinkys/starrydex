## App

app-title = StarryDex
loading = Loading...

# Header menu ("View" menu)
view = View
about = About
settings = Settings

# About page
app-info = This application has been made for learning purposes.
pokeapi-text = This application uses PokéApi and its resources.
nintendo-text = Pokémon and Pokémon character names are trademarks of Nintendo.
repository = Repository
support = Support


## Settings page

# Appearance section
appearance = Appearance
theme = Theme
match-desktop = Match desktop
dark = Dark
light = Light
view-mode = View Mode
view-mode-responsive = Responsive
view-mode-manual = Manual
pokemon-per-row = Pokémon per row
pokemon-per-page = Pokémon per page

# Pokédex section
pokedex-section = Pokédex
show-forms = Show alternate forms
show-forms-description = List megas, regional forms and other varieties as separate entries
type-filter-mode = Type filtering mode
exclusive = Exclusive
inclusive = Inclusive

# Other section (cache and bundled data)
other = Other
renew-cache = Renew Cache
renew-cache-description = Re-extracts the bundled Pokédex data, sprites and cries
renew-cache-button = Renew
data-info = Data: { $pokemon } Pokémon, { $species } species, { $moves } moves · generated { $date }


## Main page: Pokémon list

# Toolbar: search and sorting
search-placeholder = Search by name or number
sort-number = Number
sort-name = Name
reverse-order = Reverse order
order-ascending = ↑ Asc
order-descending = ↓ Desc
clear-all = Clear search and filters

# Empty results
no-results = No Pokémon found
no-results-hint = Try another search or remove some filters.

# Footer: result count and pagination
results-range = { $start }–{ $end } of { $total } Pokémon
results-none = No Pokémon
page-of = Page { $page } of { $pages }


## Filters drawer

filters-page = Filters
reset-filters = Reset
results-count = { $count ->
    [one] 1 Pokémon matches
   *[other] { $count } Pokémon match
}

# Sections
type-filters = Filter by Type
type-filter-mode-hint = The type filtering mode (match any/all selected types) can be changed in Settings.
generation-filters = Filter by Generation
category-filters = Filter by Category
stats-filter = Filter by Pokémon Stats
minimum-poke-stats = Minimum base stat total


## Pokémon details page: header and tabs

back = Back
back-to-list = Pokédex
previous-pokemon = Previous Pokémon
next-pokemon = Next Pokémon

# Tab bar
tab-overview = Overview
tab-stats = Stats
tab-moves = Moves
tab-evolution = Evolution
tab-locations = Locations
tab-forms = Forms

# Game picker (Moves and Locations tabs)
game = Game
all-games = All games


## Details › Overview tab

# Sprite card: view toggles and cries
shiny = Shiny
back-sprite = Back
female = Female
cry = Cry
cry-legacy = Classic cry

# Pokédex entry card
pokedex-entry = Pokédex entry
entry-from = From
no-flavor-text = No Pokédex entry available...

# Profile card
profile = Profile
height-label = Height
weight-label = Weight
habitat = Habitat
color = Color
shape = Shape
form = Form
alternate-form = Alternate form

# Abilities card
pokemon-abilities = Abilities
hidden-ability = Hidden

# Training card
training = Training
ev-yield = EV yield
base-exp = Base experience
catch-rate = Catch rate
base-friendship = Base friendship
growth-rate = Growth rate
exp-to-100 = { $exp } exp. to Lv. 100

# Breeding card
breeding = Breeding
egg-groups = Egg groups
gender = Gender
genderless = Genderless
egg-cycles = Egg cycles
egg-cycles-value = { $cycles } (~{ $steps } steps)

# Other cards
regional-numbers = Regional Pokédex numbers
held-items = Wild held items
appears-in = Appears in


## Details › Stats tab

# Base stats card
poke-stats = Base stats
stat = Stat
base = Base
min-100 = Min
max-100 = Max
total = Total
stat-range-hint = Min/Max: stat range at level 100 (0 IVs/0 EVs/hindering nature to 31 IVs/252 EVs/beneficial nature).

# Type defenses card
type-defenses = Type defenses
weak-4x = Takes 4× damage from
weak-2x = Takes 2× damage from
resist-half = Takes ½ damage from
resist-quarter = Takes ¼ damage from
immune = Immune to

# Previous types card
past-types = Previous types
types-until = Until { $generation }


## Details › Moves tab

no-moves = No move data available...

# Table columns
learned = Learned
move = Move
category = Category
power-short = Power
accuracy-short = Acc.
pp = PP

# "Learned" column values
level-short = Lv. { $level }
evolve-short = Evolve

# Learn methods (table titles)
level-up = Level up
tm = TM/HM
egg = Egg
tutor = Tutor
form-change = Form change
other-method = Other

# Damage classes
physical = Physical
special = Special
status = Status

# Expanded move details
target = Target
priority = Priority
introduced = Introduced
high-crit = High critical-hit ratio
drain = Drains { $value }% of damage
recoil = { $value }% recoil
flinch = { $value }% flinch chance
hits = Hits { $min }–{ $max } times


## Details › Evolution tab

poke-evo-data = Evolution chain
no-evo-data = This Pokémon does not evolve.
baby-trigger = Breeding the baby requires a parent holding { $item }

# Evolution conditions. Several can be joined with " · ", e.g. "Level up · Friendship 160+ · Daytime"
evo-level = Level { $level }
evo-level-up = Level up
evo-trade = Trade
evo-trade-for = Trade for { $species }
evo-use-item = Use { $item }
evo-holding = Holding { $item }
evo-happiness = Friendship { $value }+
evo-affection = Affection { $value }+
evo-beauty = Beauty { $value }+
evo-day = Daytime
evo-night = Nighttime
evo-dusk = Dusk
evo-female = Female
evo-male = Male
evo-knowing = Knowing { $name }
evo-knowing-type = Knowing a { $name }-type move
evo-at = At { $place }
evo-party-species = With { $name } in the party
evo-party-type = With a { $name }-type Pokémon in the party
evo-atk-gt-def = Attack > Defense
evo-atk-lt-def = Attack < Defense
evo-atk-eq-def = Attack = Defense
evo-used-move = Use { $name } { $count } times
evo-steps = Walk { $count } steps
evo-damage = Take { $count } damage
evo-rain = While raining
evo-upside-down = Console upside down
evo-multiplayer = In Union Circle
evo-chance = { $value }% chance


## Details › Locations tab

no-encounter-info = This Pokémon can't be found in the wild. Evolve, breed or trade to get it.


## Details › Forms tab

# Forms card
forms = Forms
no-forms = This Pokémon has no other forms.

# Sprites card
sprites = Sprites
no-sprites = No sprites available.
sprite-front = Front
sprite-back = Back
sprite-front-shiny = Front (shiny)
sprite-back-shiny = Back (shiny)
sprite-front-female = Front (female)
sprite-back-female = Back (female)
sprite-front-shiny-female = Front (shiny, female)
sprite-back-shiny-female = Back (shiny, female)


## Shared Pokémon vocabulary
## Used across several pages (list, filters, details).

# Types
normal = Normal
fire = Fire
water = Water
electric = Electric
grass = Grass
ice = Ice
fighting = Fighting
poison = Poison
ground = Ground
flying = Flying
psychic = Psychic
bug = Bug
rock = Rock
ghost = Ghost
dragon = Dragon
dark = Dark
steel = Steel
fairy = Fairy

# Stats: full names
hp = HP
attack = Attack
defense = Defense
sp-a = Special Attack
sp-d = Special Defense
spd = Speed
accuracy = Accuracy
evasion = Evasion

# Stats: short names (charts, compact tables, EV yield)
atk-short = Atk
def-short = Def
spa-short = SpA
spd-short = SpD
spe-short = Spe

# Generations
gen-i = Gen I
gen-ii = Gen II
gen-iii = Gen III
gen-iv = Gen IV
gen-v = Gen V
gen-vi = Gen VI
gen-vii = Gen VII
gen-viii = Gen VIII
gen-ix = Gen IX
unknown = Unknown

# Categories (filters and profile tags)
category-regular = Regular
category-baby = Baby
category-legendary = Legendary
category-mythical = Mythical

# Growth rates
growth-slow = Slow
growth-medium = Medium Fast
growth-fast = Fast
growth-medium-slow = Medium Slow
growth-erratic = Erratic
growth-fluctuating = Fluctuating

# Egg groups
egg-monster = Monster
egg-water1 = Water 1
egg-water2 = Water 2
egg-water3 = Water 3
egg-bug = Bug
egg-flying = Flying
egg-field = Field
egg-fairy = Fairy
egg-grass = Grass
egg-human-like = Human-Like
egg-mineral = Mineral
egg-amorphous = Amorphous
egg-ditto = Ditto
egg-dragon = Dragon
egg-undiscovered = Undiscovered
