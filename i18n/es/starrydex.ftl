## App

app-title = StarryDex
loading = Cargando...

# Header menu ("View" menu)
view = Ver
about = Acerca de
settings = Ajustes

# About page
app-info = Esta aplicación ha sido desarrollada con el fin de aprender.
pokeapi-text = Esta aplicación hace uso de PokéApi y sus recursos.
nintendo-text = Los Pokémon y sus nombres pertenecen a Nintendo.
repository = Repositorio
support = Soporte


## Settings page

# Appearance section
appearance = Apariencia
theme = Tema
match-desktop = Seguir sistema
dark = Oscuro
light = Claro
view-mode = Modo de vista
view-mode-responsive = Adaptable
view-mode-manual = Manual
pokemon-per-row = Pokémon por fila
pokemon-per-page = Pokémon por página

# Pokédex section
pokedex-section = Pokédex
show-forms = Mostrar formas alternativas
show-forms-description = Lista megaevoluciones, formas regionales y otras variedades como entradas separadas
cry-volume = Volumen de los gritos
type-filter-mode = Modo de filtrado por tipos
exclusive = Exclusivo
inclusive = Inclusivo

# Other section (cache and bundled data)
other = Otros
renew-cache = Renovar Cache
renew-cache-description = Vuelve a extraer los datos, sprites y gritos incluidos
renew-cache-button = Renovar
data-info = Datos: { $pokemon } Pokémon, { $species } especies, { $moves } movimientos · generados el { $date }


## Main page: Pokémon list

# Toolbar: search and sorting
search-placeholder = Buscar por nombre o número
sort-number = Número
sort-name = Nombre
reverse-order = Invertir orden
order-ascending = ↑ Asc
order-descending = ↓ Desc
clear-all = Borrar búsqueda y filtros

# Empty results
no-results = No se encontró ningún Pokémon
no-results-hint = Prueba otra búsqueda o quita algunos filtros.

# Footer: result count and pagination
results-range = { $start }–{ $end } de { $total } Pokémon
results-none = Ningún Pokémon
page-of = Página { $page } de { $pages }


## Filters drawer

filters-page = Filtros
reset-filters = Restablecer
results-count = { $count ->
    [one] 1 Pokémon coincide
   *[other] { $count } Pokémon coinciden
}

# Sections
type-filters = Fitrar por Tipo
type-filter-mode-hint = El modo de filtrado por tipo (cualquiera/todos los tipos seleccionados) se cambia en Ajustes.
generation-filters = Filtrar por Generación
category-filters = Filtrar por categoría
stats-filter = Filterar por Estadísticas
minimum-poke-stats = Total mínimo de estadísticas base


## Pokémon details page: header and tabs

back = Atras
back-to-list = Pokédex
previous-pokemon = Pokémon anterior
next-pokemon = Pokémon siguiente

# Tab bar
tab-overview = General
tab-stats = Estadísticas
tab-moves = Movimientos
tab-evolution = Evolución
tab-locations = Ubicaciones
tab-forms = Formas

# Game picker (Moves and Locations tabs)
game = Juego
all-games = Todos los juegos


## Details › Overview tab

# Sprite card: view toggles and cries
shiny = Variocolor
back-sprite = Espalda
female = Hembra
cry = Grito
cry-legacy = Grito clásico

# Pokédex entry card
pokedex-entry = Entrada de la Pokédex
entry-from = De
no-flavor-text = No hay entrada de la Pokédex disponible...

# Profile card
profile = Perfil
height-label = Altura
weight-label = Peso
habitat = Hábitat
color = Color
shape = Forma corporal
form = Forma
alternate-form = Forma alternativa

# Abilities card
pokemon-abilities = Habilidades
hidden-ability = Oculta

# Training card
training = Entrenamiento
ev-yield = EVs que otorga
base-exp = Experiencia base
catch-rate = Ratio de captura
base-friendship = Amistad base
growth-rate = Ritmo de crecimiento
exp-to-100 = { $exp } exp. hasta Nv. 100

# Breeding card
breeding = Crianza
egg-groups = Grupos huevo
gender = Sexo
genderless = Sin sexo
egg-cycles = Ciclos de huevo
egg-cycles-value = { $cycles } (~{ $steps } pasos)

# Other cards
regional-numbers = Números en Pokédex regionales
held-items = Objetos que lleva en estado salvaje
appears-in = Aparece en


## Details › Stats tab

# Base stats card
poke-stats = Estadísticas base
stat = Estadística
base = Base
min-100 = Mín
max-100 = Máx
total = Total
stat-range-hint = Mín/Máx: rango a nivel 100 (0 IVs/0 EVs/naturaleza desfavorable a 31 IVs/252 EVs/naturaleza favorable).

# Type defenses card
type-defenses = Defensas por tipo
weak-4x = Recibe 4× daño de
weak-2x = Recibe 2× daño de
resist-half = Recibe ½ daño de
resist-quarter = Recibe ¼ daño de
immune = Inmune a

# Previous types card
past-types = Tipos anteriores
types-until = Hasta { $generation }


## Details › Moves tab

no-moves = No hay datos de movimientos...

# Table columns
learned = Aprende
move = Movimiento
category = Categoría
power-short = Poten.
accuracy-short = Prec.
pp = PP

# "Learned" column values
level-short = Nv. { $level }
evolve-short = Evoluc.

# Learn methods (table titles)
level-up = Subir de nivel
tm = MT/MO
egg = Huevo
tutor = Tutor
form-change = Cambio de forma
other-method = Otro

# Damage classes
physical = Físico
special = Especial
status = Estado

# Expanded move details
target = Objetivo
priority = Prioridad
introduced = Introducido
high-crit = Índice de golpe crítico alto
drain = Absorbe el { $value }% del daño
recoil = { $value }% de daño de retroceso
flinch = { $value }% de hacer retroceder
hits = Golpea { $min }–{ $max } veces


## Details › Evolution tab

poke-evo-data = Cadena evolutiva
no-evo-data = Este Pokémon no evoluciona.
baby-trigger = Para criar al bebé, un progenitor debe llevar { $item }

# Evolution conditions. Several can be joined with " · ", e.g. "Level up · Friendship 160+ · Daytime"
evo-level = Nivel { $level }
evo-level-up = Subir de nivel
evo-trade = Intercambio
evo-trade-for = Intercambio por { $species }
evo-use-item = Usar { $item }
evo-holding = Llevando { $item }
evo-happiness = Amistad { $value }+
evo-affection = Afecto { $value }+
evo-beauty = Belleza { $value }+
evo-day = De día
evo-night = De noche
evo-dusk = Al atardecer
evo-female = Hembra
evo-male = Macho
evo-knowing = Conociendo { $name }
evo-knowing-type = Conociendo un movimiento de tipo { $name }
evo-at = En { $place }
evo-party-species = Con { $name } en el equipo
evo-party-type = Con un Pokémon de tipo { $name } en el equipo
evo-atk-gt-def = Ataque > Defensa
evo-atk-lt-def = Ataque < Defensa
evo-atk-eq-def = Ataque = Defensa
evo-used-move = Usar { $name } { $count } veces
evo-steps = Caminar { $count } pasos
evo-damage = Recibir { $count } de daño
evo-rain = Mientras llueve
evo-upside-down = Consola boca abajo
evo-multiplayer = En el Círculo Unión
evo-chance = { $value }% de probabilidad


## Details › Locations tab

no-encounter-info = Este Pokémon no se encuentra en estado salvaje. Evoluciónalo, críalo o intercámbialo.


## Details › Forms tab

# Forms card
forms = Formas
no-forms = Este Pokémon no tiene otras formas.

# Sprites card
sprites = Sprites
no-sprites = No hay sprites disponibles.
sprite-front = Frente
sprite-back = Espalda
sprite-front-shiny = Frente (variocolor)
sprite-back-shiny = Espalda (variocolor)
sprite-front-female = Frente (hembra)
sprite-back-female = Espalda (hembra)
sprite-front-shiny-female = Frente (variocolor, hembra)
sprite-back-shiny-female = Espalda (variocolor, hembra)


## Shared Pokémon vocabulary
## Used across several pages (list, filters, details).

# Types
normal = Normal
fire = Fuego
water = Agua
electric = Eléctrico
grass = Planta
ice = Hielo
fighting = Lucha
poison = Veneno
ground = Tierra
flying = Volador
psychic = Psíquico
bug = Bicho
rock = Roca
ghost = Fantasma
dragon = Dragón
dark = Siniestro
steel = Acero
fairy = Hada

# Stats: full names
hp = Vida
attack = Ataque
defense = Defensa
sp-a = Ataque Especial
sp-d = Defensa Especial
spd = Velocidad
accuracy = Precisión
evasion = Evasión

# Stats: short names (charts, compact tables, EV yield)
atk-short = Ata
def-short = Def
spa-short = AtE
spd-short = DfE
spe-short = Vel

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
unknown = Desconocida

# Categories (filters and profile tags)
category-regular = Normal
category-baby = Bebé
category-legendary = Legendario
category-mythical = Singular

# Growth rates
growth-slow = Lento
growth-medium = Medio
growth-fast = Rápido
growth-medium-slow = Parabólico
growth-erratic = Errático
growth-fluctuating = Fluctuante

# Egg groups
egg-monster = Monstruo
egg-water1 = Agua 1
egg-water2 = Agua 2
egg-water3 = Agua 3
egg-bug = Bicho
egg-flying = Volador
egg-field = Campo
egg-fairy = Hada
egg-grass = Planta
egg-human-like = Humanoide
egg-mineral = Mineral
egg-amorphous = Amorfo
egg-ditto = Ditto
egg-dragon = Dragón
egg-undiscovered = Desconocido
