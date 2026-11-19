# Parkbound — uitgebreide spelershandleiding

Versie 0.99.1 · Windows · volledig offline te lezen · Nederlands met de Engelse knopnamen uit het spel.

## 1. Wegwijs in deze handleiding

Parkbound is een parkbouw- en managementspel. Je legt een bereikbaar park aan, bouwt attracties, verzorgt je bezoekers en houdt inkomsten en uitgaven in evenwicht. In een campagne komen daar concrete doelen en soms een tijdslimiet bij. In Sandbox bepaal je zelf je opdracht.

Open in het spel **Help** met **H** of **F1** en kies het documenticoon **Handleiding (NL) - open in browser**. De toelichting verschijnt wanneer je boven het icoon blijft hangen of het met het toetsenbord selecteert. Je kunt ook `docs/HANDLEIDING_NL.html` naast `Parkbound.exe` dubbelklikken. Daarvoor is geen internet nodig. Deze Markdown-versie bevat dezelfde uitleg.

De browserversie heeft links naar elk hoofdstuk en subonderwerp, een zoekvak, een knop om de zoekopdracht te wissen en een afdrukknop. Zoek bijvoorbeeld op “wachtrij”, “voorraad”, “rente” of “hoogte”. Zoeken toont de bijpassende hoofdstukken en markeert de gevonden woorden. Klik op een hoofdstuk om de volledige tekst daarvan te zien. Met **Ctrl+F** zoek je ook via je browser. Met **/** ga je naar het zoekvak; **Escape** wist daar de filter. Gebruik de terugknop van je browser om na een verwijzing terug te gaan.

**Begin hier:** [eerste speelbare park](#3-je-eerste-speelbare-park-stap-voor-stap), [alle menu's](#5-schermindeling-en-menus), [individuele instellingen](#9-een-attractie-afzonderlijk-beheren), [geld verdienen](#15-financien-en-winst), [campagne spelen](#19-de-campagne-zes-scenarios), [problemen oplossen](#23-problemen-oplossen).

De gids beschrijft de beschikbare spelversie. Menu-iconen hebben een naam bij hover of toetsenbordfocus. Een grijze knop of rode bouwpreview heeft doorgaans een concrete reden: onderzoek, geld, terrein, verbinding, bedrijfsstatus of scenarioregels. Lees die melding voordat je opnieuw klikt. Historische technische documenten in de map docs kunnen oudere aantallen en versies noemen; deze gids behandelt de huidige spelerswerkwijze.

## 2. Installeren, starten en een spel kiezen

### Windows installeren of een portable pakket gebruiken

1. Gebruik de Parkbound Windows-installer of pak het volledige portable zipbestand uit.
2. Bewaar bij het portable pakket ook de mappen **assets**, **designs** en **docs**. Alleen het exe-bestand kopiëren laat onderdelen van het pakket achter.
3. Start Parkbound via de snelkoppeling of `Parkbound.exe`.
4. Het spel gebruikt standaard een randloos scherm. Open **Settings** om scherm, weergavemodus en interfaceschaal aan te passen.
5. Is het scherm onbruikbaar? Druk **F11** voor venstermodus op het huidige scherm met automatische interfaceschaal. Als noodstart bestaat ook `Parkbound.exe --safe-windowed`.

### Wat betekenen de keuzes in het hoofdmenu?

| Keuze | Wat er gebeurt | Geschikt voor |
| --- | --- | --- |
| Play Sunhaven Gardens | Een nieuwe, aangeklede demonstratie met zeven werkende attracties | Rondkijken en leren beheren |
| Continue latest park | Je laatst opgeslagen park laden | Verder spelen |
| New Park | Een nieuw Sandbox-park instellen | Zelf bouwen |
| Campaign | Een scenario met doelen en ontgrendelingen starten | Een gerichte uitdaging |
| Load Park | Een opgeslagen park, of de vorige geldige backup ervan, kiezen | Meerdere parken beheren |
| Scenarios and editor | Eigen scenario's maken, importeren en spelen | Zelf opdrachten ontwerpen |
| Settings | Beeld, geluid, besturing en toegankelijkheid | Het spel prettig instellen |

Het hoofdmenu behoudt tekstknoppen. Tijdens het spelen zijn veel acties iconen. Houd de muis even stil boven een icoon om naam, sneltoets en eventuele prijs te lezen. **Tab** en **Shift+Tab** verplaatsen de focus; **Enter** bedient de geselecteerde knop. **Escape** gaat een scherm terug.

### Sandbox instellen

Geef je park een naam. Kies een kaart van **32, 64, 128 of 256 tegels per zijde**, een beschikbaar landschap en je startregels. De 32-tegelvariant heeft vlak terrein en startervoorzieningen. Grotere parken beginnen leeg en kunnen vlak of met glooiingen worden gemaakt. De geldkeuzes zijn $500, $1.000, $5.000 en $10.000. Je kunt onderzoek, bezoekersinstroom, leningen en het toegangsbeleid instellen.

Kies voor een eerste eigen park **32**, vlak terrein en een ruim budget. Wil je zonder onderzoekssloten oefenen, kies dan alle inhoud beschikbaar. Druk **Create Park**; bij het openen van dit scherm is dat de eerste toetsenbordkeuze. Een Sandbox heeft geen verplichte eindscore of deadline. Een groter park geeft ruimte, maar vereist ook een zelf gebouwde ingang en meer verbindingswerk.

## 3. Je eerste speelbare park stap voor stap

### Route A: leren in Sunhaven Gardens

1. Kies **Play Sunhaven Gardens**. Dit start iedere keer een nieuwe demonstratie; het laadt geen eerdere bezoekronde.
2. Druk **Space** om te pauzeren. Bekijk de zeven attracties, serviceboulevard en ingang.
3. Sleep met rechts om rond te kijken. Zoom boven de kaart met het muiswiel. Klik kort met rechts op een gebouw om juist dát object te beheren.
4. Open **Guests** en bekijk tevredenheid en behoeften. Selecteer een bezoeker en kies **Find** om die terug te vinden.
5. Open **Management and economy → Overview** en daarna **History**. Vergelijk verkoopopbrengsten met lopende kosten.
6. Klik met rechts op een service. Controleer of hij open is en een redelijke prijs heeft; controleer voorraad onder **Supplies**.
7. Hervat met **Space**. Kijk hoe bezoekers via wachtrijen instappen en na de rit naar de openbare paden terugkeren.
8. Sluit vensters en druk **F5** om je eigen voortgang op te slaan. Laad die later via **Continue latest park** of **Load Park**.

Sunhaven heeft Sunburst Coaster, Willow Falls, Sunhaven Railway, Garden Carousel, Blossom Cups, Golden Wheel en Marigold Voyager. Daarnaast zijn alle acht services en een team met de vijf personeelsrollen aanwezig. De demonstratie heeft geen campagnedeadline en alle onderzoek is beschikbaar. Het openingsbudget is $15.000, maar bouwkosten en reeds gespeelde activiteit zijn al in de getoonde kas verwerkt. Verwacht dus geen kassaldo van exact $15.000.

### Route B: je eigen eerste park bouwen

1. Maak een Sandbox. Pauzeer direct. Gebruik desgewenst **Help → First park guide** voor de ingebouwde zevenstappengids.
2. Zoek de openbare paden. Op een lege grote kaart: bouw vanaf de westelijke rand via de beschikbare landstrook naar je bouwgebied.
3. Kies een openbaar pad als bezoekersingang via **Visitor flow**. Controleer dat instroom is ingeschakeld en het park open is.
4. Kies in **Build → Catalogue** een beschikbare rustige attractie, bijvoorbeeld een carousel. Plaats haar op vrij, eigen terrein; laat rondom ruimte voor toegang en uitgang.
5. Open met rechts de attractie en kies **Access**. Maak de ingang aan de attractie, verbind die met een wachtrij en sluit de andere kant van de wachtrij aan op het openbare pad.
6. Maak een afzonderlijke uitgang. Het openbare pad naast die uitgang moet terug naar het park leiden. Gebruik de suggesties in **Access** als een verbinding ontbreekt.
7. Open **Operations**. Los alle gemelde problemen op en open de attractie. Begin met een bescheiden ticketprijs.
8. Bouw drank, eten en toiletten met hun bedieningszijde aan een openbaar pad. Controleer opening, prijs en voorraad.
9. Plaats een paar bankjes en afvalbakken. Houd geld over voor personeel, voorraad en onderhoud.
10. Hervat. Controleer echte aankomsten, voltooide ritten en klanten in de rapporten. Alleen een bewegende attractie bewijst nog niet dat bezoekers haar kunnen bereiken.
11. Druk **F5**. Bouw pas een tweede attractie wanneer de eerste daadwerkelijk klanten ontvangt en het kostenoverzicht begrijpelijk is.

### Een bruikbare eerste speelroutine

Werk per korte periode: bouw gepauzeerd, laat daarna even spelen, controleer wat er werkelijk gebeurt en pas één probleem tegelijk aan. Kijk achtereenvolgens naar bereikbaarheid, bezoekersbehoeften, bedrijfstoestand, voorraad en geld. Sla op na een geslaagde uitbreiding en vóór een grote verbouwing. Versnellen is nuttig nadat je zeker weet dat bezoekers niet vastlopen en vaste kosten betaalbaar zijn.

## 4. Het doel en de spelklok

In Sandbox bouw je een aantrekkelijk, bereikbaar en betaalbaar park volgens je eigen doel. Voorbeelden zijn zeven verschillende attracties, tevreden bezoekers, een park zonder verdwaalde gasten of meerdere winstgevende periodes achter elkaar. De campagne geeft expliciete voorwaarden; het halen van één getal is onvoldoende als het scenario meerdere gelijktijdige doelen heeft.

**Space** pauzeert of hervat. Buiten invoervelden en speciale vensters kiezen **1**, **2** en **3** de snelheden 1×, 2× en 4×. In het Explore-menu kiezen die cijfers juist een menugroep. De vermelding “30 seconden” bij lonen, onderzoek of marketing betekent **simulatietijd**, geen tijd op de Windows-klok. Versnellen versnelt ook uitgaven, bezoekersbehoeften en deadlines.

Een instellingenvenster voor attracties of services pauzeert het park niet automatisch. Druk eerst Space als je rustig wilt lezen. De handleiding in de browser pauzeert evenmin automatisch. Een geladen park begint gepauzeerd; controleer de toestand en hervat bewust.

## 5. Schermindeling en menu's

### Header, zijbalk en Explore

Bovenaan staan de parknaam en kerncijfers. De snelkoppelingen naar financiën, attracties en **Tips** staan bij elkaar. Links staan algemene acties; de bouwsectie bevat gegroepeerde gereedschappen. Een geselecteerde gast, padtegel of attractie geeft contextinformatie. De onderste melding vertelt vaak waarom een actie niet lukt.

**Explore / F2** is de centrale route. Je hoeft niet ieder pictogram uit het hoofd te leren:

| Groep | Belangrijkste onderdelen |
| --- | --- |
| Build | Paden, wachtrijen, terrein, attracties, services, coasterbouw en ontwerpen |
| Manage | Financiën, economie, bezoekersinstroom, personeel en scenario-editor |
| Inspect | Bezoekers, attracties en services bekijken |
| Park | Doelen, parkstatistieken, opslaan en parkmenu |
| Help / Settings | Speluitleg, geluid, toegankelijkheid en weergave |

Vensters scrollen wanneer de inhoud niet past. Plaats de muis boven het venster en gebruik het wiel; boven de kaart zoomt datzelfde wiel. **Escape** sluit eerst een venster of actief gereedschap; vanuit de vrije kaart opent het het parkmenu. Een tekstveld moet eerst worden verlaten voordat een letter weer als sneltoets dient.

### Iconen lezen

De vorm geeft de actie aan: spoor/rail voor coasterbouw, slingerende lijnen voor paden, poort voor toegang, pijl naar buiten voor uitgang, gereedschap voor onderhoud, munt/grafiek voor geld, personen voor bezoekers en lamp voor Tips. Plus en min wijzigen de waarde in de direct bijbehorende groep. Controleer de hovertekst om bijvoorbeeld padhoogte van borstelgrootte te onderscheiden. Een lijstregel of naamveld blijft leesbare tekst. Het hoofdmenu, New Park en Settings behouden hun tekstkeuzes.

## 6. Camera, beeld, geluid en toegankelijkheid

### De kaart bekijken

Sleep met de **rechter- of middelste muisknop** om te pannen. Een korte rechterklik zonder slepen opent objectinstellingen. Gebruik **WASD** of de pijltjes voor camera-beweging als geen invoerveld of keuzelijst die toetsen gebruikt. **Q/E** draaien de kaart een kwartslag. Het muiswiel of **+/−** zoomt; **Home** past de kaart in beeld. **G** toont het raster; **U** wisselt ondergrondweergave voor tunnels.

Roteren helpt wanneer een hoog gebouw een ingang verbergt. Uitzoomen helpt voor het netwerk; inzoomen helpt bij hoogteverschillen en kleine poorten. Klik de zichtbare geometrie van de attractie, niet de lege ruimte naast een hoge constructie. De rechterklikselectie houdt rekening met rotatie en interfaceschaal.

### Een hogere resolutie gebruiken

Open **Menu → Settings**. Kies het scherm, Windowed, Borderless of Fullscreen en de interfaceschaal **Auto, 100%, 125%, 150% of 200%**. Een onbruikbaar grote schaal kan worden begrensd zodat bediening op het scherm blijft; Settings toont de effectieve schaal. Op een klein scherm moet je vaker door vensters scrollen. F11 herstelt de veilige vensterweergave.

### Muziek, attractiegeluiden en reacties

**F10** opent geluid. Stel Master, Music, Ambience, Rides, Alerts & interface en Guest & staff reactions afzonderlijk in. Zoom en camerapositie beïnvloeden nabijheid en stereopositie van attractiegeluiden. Waterattracties, ketting-/rijgeluiden en bezoekersreacties horen bij de betreffende activiteit. Zet Music lager als je vooral de omgeving wilt horen; zet Master op nul of gebruik Mute voor stilte.

Geluid vervaagt bij focusverlies. **Reconnect audio device** probeert de huidige Windows-uitvoer opnieuw wanneer je van apparaat wisselt of niets hoort. Korte ondertitels geven belangrijke acties ook zonder geluid weer. Muziek en omgevingsgeluid kunnen tijdens pauze doorlopen, terwijl ritactiviteit stopt.

### Toegankelijkheid en eigen toetsen

Open **Controls & accessibility** vanuit Settings of Help. Je kunt achttien spelacties een andere beschikbare toets geven. De lijst toont de actuele toewijzingen; vaste navigatie- en hersteltoetsen behouden hun rol. De referentie verderop beschrijft standaardtoetsen. **High contrast** maakt tekst en vlakken duidelijker. **Reduced motion** vermindert decoratieve beweging; voertuigen en bezoekers blijven hun werkelijke posities tonen. Ondertitels zijn afzonderlijk instelbaar.

## 7. Terrein, land en tuinen

### Eigen grond en terreinhoogte

Je kunt alleen bouwen op grond waarvoor het park rechten heeft. Grote kaarten beginnen met een centraal eigen gebied en een smalle verbinding naar de westrand. **Buy Land** koopt aansluitende tegels voor $5 per tegel. Verschillende grondkleuren en grenzen tonen het eigendom.

**Raise** en **Lower** verhogen of verlagen het terrein één stap. De borstel kan 1×1, 3×3 of 5×5 tegels bestrijken. De preview laat de echte prijs en het hele gebied zien. Verhogen/verlagen kost $1 per veranderde tegel, gras/zand schilderen $0,50 en water $2. Als één tegel ongeldig is of het geld ontbreekt, wordt de hele bewerking geweigerd. Verwijder of verplaats blokkerende bebouwing eerst.

Terrein wijzigen verplaatst een bestaande verhoogde attractie niet automatisch naar beneden. Controleer gebouwhoogte, padhoogte en grondhoogte afzonderlijk. Gebruik voor een nieuwe attractie de gewenste bouwhoogte voordat je plaatst. Voor grote wijzigingen is een nieuwe plaatsing of aangepaste coastergeometrie nodig.

### Water, bruggen en tunnels

Water vereist vlakke geschikte grond. Grondpaden en objecten kunnen het aanbrengen blokkeren. Een brug met voldoende hoogte kan boven water blijven bestaan; een nieuw grondpad op water is ongeldig. Een pad mag niet half door aarde heen lopen.

Voor een tunnel: zet **Underground view** aan, kies Pad of Queue en zet de padhoogte voldoende onder het maaiveld. Het complete profiel moet ondergronds passen; een helling vraagt extra vrije hoogte. De hint meldt **Tunnel** wanneer dat klopt. Terug in Surface view zijn begraven delen verborgen; zichtbare tunnelmonden verschijnen bij aansluitingen naar buiten. Gebruik Underground view opnieuw om ze te selecteren of wijzigen.

### Bomen, bloemen en meubilair

Plaats bomen en bloemen op vrije eigen gras- of zandtegels. Bomen kosten $2,50 en bloemen $1,50; verwijderen van betaald groen geeft de helft terug. Gratis aanwezige aankleding heeft geen oorspronkelijke aanschafwaarde om terug te betalen. Groen bij looproutes helpt de sfeer en tevredenheid, maar vervangt geen eten, drinken of rust.

Bankjes en afvalbakken horen op openbare paden. Richtingborden en hekken geven visuele structuur; ze veranderen op zichzelf de routekeuze van bezoekers niet. **Cover a path** voegt een dak toe aan een bankje. Een nieuw bankje kost $1; een bestaand bankje overdekken is gratis. Houd belangrijke ingangen zichtbaar en looproutes kort in plaats van elk vrij vak te vullen.

## 8. Paden, wachtrijen, poorten en hoogte

### Vier onderdelen die niet uitwisselbaar zijn

| Onderdeel | Functie | Waar sluit het op aan? |
| --- | --- | --- |
| Public path / Path | Openbare circulatie voor gasten en personeel | Andere openbare paden en de parkingang |
| Queue | Wachten op één specifieke attractie | Openbare toegang aan één einde, attractie-ingang aan het andere |
| Entrance / Gate | Instappen bij de geselecteerde attractie | De juiste attractie en haar eigen wachtrij |
| Exit | Uitstappen na de rit | De attractie en een openbaar pad terug het park in |

Een werkend schema is:

```text
parkingang === openbaar pad === services === openbaar parknetwerk
                    |
              wachtrij van A --> ingang A --> attractie A
                                              |
                        openbaar pad <------ uitgang A
```

Een wachtrij is geen openbaar doorgaand pad. Een service heeft een openbaar pad aan de bedieningszijde nodig. De wachtrij en poorten horen bij een geselecteerde attractie: controleer de naam voordat je ze bouwt. Gebruik bij voorkeur **Rides → Access** om de selectie en ontbrekende onderdelen te zien.

### Bouw een verbinding stap voor stap

1. Kies **Path / P** voor het openbare deel of **Queue / J** voor het wachtrijdeel.
2. Stel hoogte en vorm in voordat je klikt. Bij hellingen wisselt **C** de richting/vorm; bij poorten en services draait C de oriëntatie.
3. Plaats opeenvolgende tegels met gelijk aansluitende **randhoogten**. Twee naast elkaar liggende tegels op verschillende niveaus zijn niet automatisch verbonden.
4. Gebruik een éénstapshelling tussen opeenvolgende niveaus. De hoge zijde moet precies aansluiten op de hoge buur.
5. Plaats de ingang met **I**, gericht naar de attractie, en de uitgang met **K**, met een eigen openbaar retourpad.
6. Controleer **Access**. Los iedere rode voorwaarde op en test daarna met een echte bezoeker.

Een vlak openbaar pad kost $1; een hellend pad $1,50. Een vlakke wachtrij kost $2; een hellende wachtrij $2,50. Hover toont het bedrag voor de gekozen tegel. Ongeldige plaatsing kost niets. Paden kunnen visueel dicht bij elkaar liggen zonder te verbinden: controleer hoogte, richting, type en eigendom.

### Demolition, verwijderen en ongedaan maken

**Demolish / X** verwijdert het aangewezen onderdeel als de spelregels dat toestaan. **Delete** verwijdert een geselecteerd object. Sluit attracties en laat bezoekers uitstappen vóór een grote sloopactie. Let erop dat een verwijderd pad ook meubilair en bereikbaarheid kan beïnvloeden.

Betaalde paden geven de helft van de oorspronkelijke padprijs terug plus toepasselijke meubelteruggaven. Attracties en services tonen hun eigen geldige refund; bedragen hangen af van wat werkelijk voor dat object betaald is. Een nieuw bouwwerk plaatsen is niet gratis omdat je eerder iets sloopte: beide transacties worden afzonderlijk geboekt.

Bij gepauzeerde parkbouw kun je **Ctrl+Z** en **Ctrl+Y** gebruiken voor maximaal zestien recente bewerkingen. Hervatten, een ander park laden of wisselen wist deze tijdelijke geschiedenis. De coasterdraft heeft een eigen undo/redo. Een save bewaart het park, niet die undo-lijst. De gebruikte Tips-teller wordt niet teruggedraaid.

## 9. Een attractie afzonderlijk beheren

### Met rechts het juiste instellingenvenster openen

Klik **kort met de rechtermuisknop** op een zichtbare attractie, haar track, eigen poort of wachtrij. Het venster **Rides** opent met de eigenaar van precies dat onderdeel geselecteerd. Klik met rechts op een servicegebouw voor **Services**. Een rechterklik op leeg terrein opent niets. Houd je de knop vast en sleep je, dan verplaats je de camera en verschijnt geen instellingenvenster.

De selectie wordt niet aangenomen wanneer je ondertussen van camera, park of venster wisselt. Deze bescherming voorkomt dat de verkeerde attractie wordt geopend na slepen of laden. Een open venster bedekt de kaart; sluit het eerst voordat je daarachter een andere attractie probeert aan te klikken.

### Operations, naam en kleuren

**Operations** toont status, opening, capaciteit en de instellingen die bij het model horen. Een carousel heeft andere mogelijkheden dan een coaster of transporttrein. Kies alleen aangeboden instellingen. Wijzig voertuigaantal, treinconfiguratie of vergelijkbare uitrusting wanneer de attractie dicht, stil en leeg is. Een sluitverzoek laat reeds vertrokken ritten veilig afronden.

Pas minimumbezetting en wachttijden aan op de vraag. Hoge minimumbezetting kan in een rustig park lang lege stoelen vasthouden. Te veel treinen helpen niet als station of baanblokken onvoldoende ruimte hebben. **Reset** herstelt operationele standaardwaarden. Het paneel toont ook onderhoudstoestand, bedrijfskosten en een reden wanneer openen wordt geweigerd.

| Instelling | Betekenis | Wanneer aanpassen? |
| --- | --- | --- |
| Cars / Trains | Wagen- en treinaantal, waar ondersteund | Meer capaciteit nadat baanblokken en vraag dat toelaten |
| Station wait | Basiswachttijd aan het station | Balans tussen vlot vertrek en verzamelen van instappers |
| Minimum load | Gewenst minimum aantal passagiers | Lager bij weinig bezoekers; nul schakelt het minimum uit |
| Max extra wait | Extra wachten om minimumlading te halen | Voorkom dat een rustige rit onnodig lang blijft staan |
| Lift speed | Snelheid op liftstukken | Alleen met aandacht voor de volgende baandelen en testresultaten |
| Brake speed | Reminstelling van de baan | Aankomstsnelheden beheersen; daarna opnieuw testen |
| Cycle duration | Duur van een vaste attractiecyclus | Afweging tussen ervaring en aantal ritten per tijdseenheid |

Bij **Edit name** typ je een eigen naam en bevestig je met **Apply name**. **Restore default** herstelt de modelnaam. Onder **Colors** kies je de beschikbare verfschema's. Naam en kleur veranderen de bereikbaarheid, ritveiligheid en prijs niet.

### Ticketprijs en inspectie

Na de rechterklik blijft de gekozen attractie geselecteerd. Sluit Rides met **Use ride / Escape** om haar kaartinspecteur te zien. Gebruik daar **F6/F7** of de prijsiconen voor stappen van $0,50. **O** opent of sluit de geselecteerde attractie; **F** brengt haar in beeld. De prijzen van verschillende attracties zijn onafhankelijk. Onder **Management → Policies** bestaan daarnaast prijsinstellingen voor alle attracties tegelijk.

Let op scenarioregels: bij gratis attracties kun je niet alsnog een ticketopbrengst afdwingen. Een bezoeker die al een prijsafspraak heeft, behoudt zijn afgesproken bedrag. Nieuwe prijzen gelden voor latere afspraken. Geld wordt bij daadwerkelijk vertrek geïnd; een geannuleerde instap is geen verkochte rit.

### Access, testen en onderhoud

**Access** is de checklist voor ingang, wachtrij en uitgang. Kies een voorgestelde bouwactie, plaats de preview en open Access opnieuw voor het volgende ontbrekende onderdeel. Een preview is nog geen gebouwd pad. **Test** berekent voor een geïnstalleerde coaster een afzonderlijke testronde. Sluit en leeg de rit eerst; zie het hoofdstuk over coasterbouw.

Bij slijtage of onderhoudssluiting kun je de aangeboden **Service ride**-actie gebruiken of een bereikbare monteur inzetten. Eerst moet de rit veilig stil en leeg zijn. Los ook eventuele geld- en padproblemen op. Een monteur kan geen afgesloten eiland bereiken en een grotere kas heropent een geldgesloten attractie niet vanzelf.

## 10. De volledige attractiecatalogus

Open **Build → Catalogue**. Kies een familie, selecteer een beschikbaar model en lees prijs, footprint en beschrijving. Draai met **C** en plaats pas wanneer de hele preview geldig is. Meerdere exemplaren van een model kunnen naast elkaar bestaan met eigen naam, bezoekers en boekhouding. Sommige oudere snelbouwknoppen hebben beperktere mogelijkheden; gebruik de catalogus voor het volledige aanbod.

| Familie | Model | Geleverde capaciteit | Karakter en aandachtspunt |
| --- | --- | --- | --- |
| Gentle | Festival Carousel | 6 | Rustige draaimolen; geschikte eerste attractie |
| Gentle | Meadow Wheel | 6 | Reuzenrad met hoge gondelconstructie |
| Gentle | Whirling Cups | 12 | Zes draaiende tweepersoonskopjes |
| Gentle | Skyline Tower | 8 | Observatietoren met lift en kijkpauze |
| Thrill | Skyward Swing | 6 | Pendelgondel; vraag hangt af van voorkeuren |
| Thrill | Beacon Drop | 6 | Omhoog, vasthouden en vrije-valachtige afdaling |
| Thrill | Corsair Ship | 12 | Schommelend piratenschip |
| Water | Timber Falls | 4 | Boomstam met lift en splashdown |
| Water | River Rapids | 6 | Rondvaart in een raft op waterkanaal |
| Transport | Park Railway | 12 | Trein tussen twee stations |
| Transport | Vista Monorail | 12 | Monorail tussen twee stations |
| Coaster | Classic Coaster | 6 | Klassieke coaster met houten ondersteuning |
| Coaster | Steel Launch Coaster | 6 | Stalen baan met lanceer- en overgangsstukken |
| Coaster | Inverted Coaster | 6 | Hangende stoelen onder de baan |

Capaciteit in deze tabel hoort bij de meegeleverde configuratie; ondersteunde trein- en wagoninstellingen kunnen haar wijzigen. Niet elk baantype accepteert elk bouwstuk. De live catalogusquote is bepalend voor prijs, grondruimte en onderzoeksslot. Koop geen model alleen omdat de eerste aanschaf past: toegang, paden, voorraad, lonen en onderhoud vragen ook geld.

Vaste attracties gebruiken een volledig gereserveerde footprint en hoogte. De meegeleverde baanattracties beslaan ongeveer 10×7 tegels vóór rotatie. Laat daarom meer ruimte vrij dan alleen de zichtbare voertuigen suggereren. Transport heeft een instapstation en een bestemming: verbind de ingang bij het eerste station en de uitgang bij het andere. De lege trein keert terug naar het begin.

## 11. Coasters bouwen van station tot eerste rit

### Begin met een korte werkende baan

1. Pauzeer en kies **Coaster track / Z**. Kies **New coaster** voor een nieuwe baan; voor verbouwen selecteer je eerst de bestaande rit via Rides.
2. Kies een vrije, eigen plek en begin op grondniveau met een station. Reserveer ruimte buiten het station voor ingang, wachtrij en uitgang.
3. Laat **Follow heights** aan staan. Kies een recht stuk en gebruik **Add next piece / Enter**. De volgende preview neemt de uitgangsrichting en -hoogte over.
4. Bouw een eenvoudig rondgaand traject met bochten. Maak eerst een kleine werkende lus voordat je ingewikkelde hoogtewisselingen toevoegt.
5. Controleer dat het laatste stuk exact op richting en hoogte van het station aansluit. Alleen visueel dicht bij elkaar eindigen sluit de baan niet.
6. Lees de bouwquote en kies **Apply**. Dit installeert de draft en verwerkt de werkelijke aanschaf/refund. Alleen stukken tekenen in de draft bouwt nog geen rijdende attractie.
7. Open **Rides → Access**, maak ingang, eigen wachtrij en openbaar uitgangspad.
8. Laat de coaster gesloten en voer **Test → Run one isolated lap** uit. Los problemen met het traject op.
9. Stel een passende trein en laadregels in, open de attractie en laat één bezoeker een volledige reis maken. Sla daarna op.

### Klimmen, dalen en de volgende blokhoogte

De preview toont **Start → End**. Bij **Follow heights** begint elk nieuw stuk op het eindniveau van het vorige. Twee liftstukken achter elkaar klimmen dus vanzelf verder. Een daling start op het bereikte hoge niveau en geeft het volgende stuk een lagere starthoogte. Na een bocht wordt ook de richting gevolgd.

**PgUp/PgDn** verandert handmatig de ingangshoogte. Dat zet automatisch volgen uit, omdat je bewust van het vorige eindpunt afwijkt. Zet **Follow heights on** weer aan om terug te gaan naar het open einde. Een onderbroken draft zoekt bij opnieuw openen zijn laatste open aansluiting. Bij de bodem of hoogtebegrenzing kan de editor naar een recht stuk terugvallen; controleer altijd de preview.

Beschikbare stukken omvatten Station, Straight, linker/rechter Curve, Bank, Lift, één- of tweestaps Drop, Brakes, Launch, Camelback, Smooth rise, Smooth drop en Heartline roll. Een camelback eindigt op zijn beginhoogte met een top binnen de tegel. Smooth rise/drop verbindt niveaus geleidelijk. Een roll vraagt extra vrije hoogte boven de baan. Het gekozen baantype kan stukken beperken; een rode preview vermeldt de blokkade.

### Treinen, remblokken en testresultaten

Coasters ondersteunen één tot drie treinen met één tot drie tweezitswagens per trein, als het baanontwerp dat toelaat. Voor meerdere treinen zijn **meer blokken dan treinen** nodig en elk blok moet lang genoeg zijn voor een volledige trein. Het station en remstukken beginnen blokken. Plaats remstukken met voldoende tussenruimte. Een stilstaande trein vóór rood sein kan gewoon op een bezet volgend blok wachten.

Wijzig de vloot alleen als de attractie gesloten, gestopt en leeg is. De minimumlading geldt voor de trein bij het station. Sluiten laat vertrokken treinen terugkomen en uitstappen. Waterbanen hebben hun eigen beperkte voertuigconfiguratie; transport gebruikt één trein en een bestemming.

De test toont snelheden, hoogte, kenmerken, geschatte krachten en layoutbeoordelingen. Het is een modelberekening; wachtrijen, passagiersbelading en blokwachttijd maken geen deel uit van die geïsoleerde ronde. Wijzigingen aan baan of bedrijfsinstellingen maken eerdere resultaten verouderd. Test opnieuw en beoordeel daarna echte bezoekersreacties. De test verandert geen parkklok, gasten of kassaldo.

### Bestaande banen veilig aanpassen

Sla eerst op. Sluit de juiste coaster en wacht tot alle bezoekers eruit zijn. Open zijn draft, wijzig enkele stukken, controleer prijs en verbindingen, pas toe en test opnieuw. **Next coaster** wisselt tussen opgeslagen baanwerkruimtes; controleer de naam vóór Apply. Er kunnen maximaal acht coasterrecords naast de vaste attracties bestaan. De tijdelijke draftgeschiedenis is niet dezelfde geschiedenis als gepauzeerde terreinbouw.

## 12. Coasterontwerpen bewaren en delen

**Designs / Ctrl+D** opent de ontwerpbibliotheek. Een ontwerp bewaart baangeometrie, geen compleet park, bezoekersbestand of volledige bedrijfsvoering. Bewaar je huidige draft met een herkenbare naam. De bibliotheek kan zoeken, voorvertonen, hernoemen, een gekozen ontwerp bijwerken, importeren en exporteren.

Het pakket bevat **Pocket Oval**, **Meadow Winder** en **Meadow Climb** in de map designs. Importeer een `.parktrack.json`, selecteer het, pas positie, rotatie en hoogte aan en bekijk de preview. **Load into draft** laadt het ontwerp; **Apply** bouwt het werkelijk. Sluit en leeg een bestaande attractie vóór vervanging. Een verplaatst station kan nieuwe poorten en paden nodig hebben.

De oorspronkelijke plaatsingshoogte van een meegeleverd ontwerp hoeft niet grondniveau te zijn. Pas die bewust aan; Follow heights bewaakt daarna de onderlinge aansluiting. Controleer vrij terrein, tunnels, onderzoek, cash en overige bebouwing. Exporteer een geslaagd eigen ontwerp en test import in een ander park als je het wilt delen. Een gedeeld bestand ontgrendelt niet automatisch alle onderzoek van de ontvangende speler.

## 13. Services, prijzen en voorraad

### Acht voorzieningen met een eigen taak

| Service | Bouwprijs | Standaard bezoekprijs | Belangrijkste effect |
| --- | --- | --- | --- |
| Food | $120 | $2 | Vermindert honger |
| Drinks | $80 | $1 | Vermindert dorst |
| Toilets | $100 | Gratis | Vermindert toiletbehoefte |
| Rest | $140 | Gratis | Herstelt energie en helpt bij misselijkheid |
| Ice cream | $110 | $1,50 | Helpt tegen honger en dorst |
| Souvenirs | $160 | $4 | Voldoet aan souvenirlust |
| Information | $90 | Gratis | Geeft bezoekers parkinformatie |
| First aid | $180 | Gratis | Helpt misselijke en vermoeide bezoekers |

Scenario's en bestaande parken kunnen andere ingestelde prijzen hebben; Sunhaven gebruikt bijvoorbeeld lagere prijzen voor enkele services. De tabel geeft modelstandaarden, geen verplicht tarief. Gratis aanbieden voorkomt niet dat voorraad en exploitatie geld kosten.

### Eén service instellen

1. Klik met rechts op het servicegebouw. Controleer de naam en het model in **Services**.
2. Zet de service open of dicht en wijzig de prijs met de beschikbare plus/min-knoppen; F6/F7 werkt in dit venster ook voor prijsstappen.
3. Controleer klanten, verkoop, kosten en de bedieningszijde. Een naastliggend pad aan de verkeerde zijde is onvoldoende.
4. Ga voor voorraad naar **Management and economy → Supplies**. Selecteer dezelfde service.
5. Controleer voorraad, automatisch bestellen en geld. Bestel indien nodig twintig eenheden of schakel automatisch bestellen in.
6. Hervat en controleer dat het aantal bediende klanten werkelijk oploopt.

### Hoe voorraad werkt

In een park met management heeft elke service maximaal honderd verbruikseenheden. Ook toiletten, rust, informatie en eerste hulp verbruiken benodigdheden. Automatisch bestellen koopt twintig eenheden wanneer een open service onder vijf eenheden komt. De hele bestelling moet betaalbaar zijn. Bij geldgebrek wordt later opnieuw geprobeerd zolang automatisch bestellen aan staat.

Voorraad wordt betaald bij levering. Een afgeronde bediening verbruikt een eenheid en rekent de afgesproken verkoopprijs af. Een prijswijziging verandert een al gemaakte afspraak niet. Zonder voorraad kun je geen normale serviceverkoop blijven draaien. Sluiten stopt automatisch bestellen en lopende exploitatiekosten, maar lost de behoefte van bezoekers niet op.

Het rapport boekt ingekochte voorraad als uitgave zodra je haar koopt. Daardoor kan een periode met een grote bestelling tijdelijk verlies laten zien, terwijl de voorraad later nog klanten bedient. Sloop wist die historische uitgave niet en betaalt ongebruikte voorraad niet automatisch als nieuwe verkoop terug.

## 14. Bezoekers begrijpen en helpen

### Individuele gasten en groepen

**Guests / L** opent de lijst. Selecteer een persoon voor geld, tevredenheid, huidige activiteit, voorkeuren, behoeften en beschikbare reis-/groepsinformatie. **Find** centreert de kaart op die persoon. Met de pagina- en filterknoppen vind je ook een specifieke probleemgroep.

Nieuw aankomende bezoekers kunnen drietallen vormen met familie-, sensatie- of rustige voorkeuren. Ze betalen individueel en kunnen verschillende betaalbare activiteiten kiezen, maar groeperen zich tussendoor en vertrekken samen. Een blij vertrokken groep kan later opnieuw proberen te komen; dat is geen gegarandeerde onmiddellijke verkoop. Bezoekers hebben een eindige portemonnee. Een populair park met te hoge prijzen kan dus toch weinig herhaalritten verkopen.

### Parkingang en toegangsbeleid instellen

Selecteer op de kaart een geschikt openbaar pad, open **Visitor flow** en kies **Set entry**. Controleer de gemelde verbinding. **Toggle arrivals** regelt nieuwe aankomstpogingen; **Open / close park** regelt of het park bezoekers toelaat. De toegangsprijs heeft eigen min- en plusknoppen. Het beleid kan vrije toegang, gratis attracties of flexibel prijzen voorschrijven. Een scenario kan dat beleid vastzetten.

Binnenkomende bezoekers beoordelen de entree op bereidheid en beschikbare portemonnee. Een hoge entree laat minder geld over voor ritten en services. Een dicht park voorkomt nieuwe toelating maar verwijdert bestaande bezoekers niet onmiddellijk. **Send guest home** vraagt de geselecteerde gast te vertrekken; een lopende activiteit en de route naar buiten blijven relevant. Gebruik vertrek niet om een ontbrekend uitgangspad te verbergen.

### Behoeften, tevredenheid en beoordelingen

Honger, dorst, toiletbehoefte, vermoeidheid, misselijkheid en andere behoeften groeien tijdens het bezoek. Warmte verhoogt dorst. Zware ritten kunnen de behoefte aan rust of eerste hulp vergroten. Bezoekers kiezen mede op betaalbaarheid, voorkeur, bereikbaarheid en hun huidige toestand. Zorg voor variatie in attracties én services.

Tevredenheid is niet hetzelfde als attractiewaardering. Een gast kan een rit waarderen en toch ongelukkig zijn door dorst of gebrek aan rust. Groen helpt de sfeer; entertainers helpen tevredenheid; geen van beide vervangt een bereikbare toiletservice. Bij aanhoudend lage tevredenheid besluiten gasten te vertrekken. Verlaag zo nodig een prijs, verkort de route of voeg een ontbrekende voorziening toe.

### Filters en trendgrafieken gebruiken

De filters **All**, **Urgent**, **Unhappy** en **Lost** tonen alle gasten, urgente behoeften, ontevreden gasten en verdwaalde gasten. Urgent betekent minstens één van honger, dorst of toiletbehoefte op 70 of hoger. Unhappy betekent tevredenheid onder 40. Een gast kan in meer dan één categorie passen.

**Park stats** toont trends en actuele gemiddelden. De bezoekersgrafiek bemonstert tijdens het spelen en toont maximaal 24 metingen met intervallen van dertig simulatieseconden. Laden begint een nieuwe sessiegrafiek; dat betekent niet dat gastgegevens verloren zijn. Kijk of je ingreep de volgende metingen verbetert en vergelijk niet één momentopname met een hele verkoopgeschiedenis.

## 15. Financiën en winst

### Vier cijfers die je uit elkaar moet houden

| Cijfer | Betekenis | Veelgemaakte vergissing |
| --- | --- | --- |
| Cash | Nu beschikbaar geld | Een lening aanzien voor verdiende winst |
| Sales / Revenue | Ontvangsten uit bezoekersverkopen | Omzet aanzien voor winst |
| Operating result | Verkoop minus geboekte lopende kosten | Een druk park automatisch winstgevend noemen |
| Park value / assets | Boekwaarde van betaalde aanwezige bouw | Verwachten dat die waarde direct als geld beschikbaar is |

**Finance / N** geeft kasstromen, parksamenvatting, leningen, parkwaarde en transacties. De Management-overzichten geven aanvullend trends en concrete problemen. Kosten blijven historisch zichtbaar wanneer een rit of service wordt gesloopt. De recente transactielijst is begrensd; oudere bedragen blijven in totalen en eigenaarrekeningen bestaan.

### Van verkoop naar het kassaldo

Een praktische aansluiting is: startbudget plus bedrijfsresultaat, plus sloopteruggaven, min bouwuitgaven, plus opgenomen leningen, min aflossingen en betaalde rente geeft de kas. Exploitatie omvat onder andere attractie-/servicekosten, gekochte voorraad, lonen, onderzoek en advertenties. De rapporten scheiden die soorten zodat je geen verkoop hoeft te raden uit het saldo.

Bekijk **Management → History** om opeenvolgende periodes te vergelijken. Er zijn maximaal zestig financiële momentopnamen, één per simulatieminuut. Een periode met bouw- of voorraaduitgaven kan bewust negatief zijn. Beoordeel daarna of de uitbreiding voldoende extra klanten oplevert. Houd altijd een buffer voor betalingen die vóór de volgende kaartverkoop komen.

### Een park rendabeler maken

1. Los eerst ontbrekende verbindingen en gesloten verkooppunten op. Onbereikbare capaciteit verdient niets.
2. Bekijk per attractie en service het werkelijk aantal klanten en de bedrijfskosten.
3. Controleer of bezoekers prijzen afwijzen of simpelweg geen geld meer hebben. Pas één prijs tegelijk aan.
4. Sluit onnodige verliesgevende voorzieningen tijdelijk of bouw compacter; laat essentiële behoeften wel bereikbaar bedienen.
5. Onderzoek pas wat je binnenkort kunt betalen en plaatsen. Vermijd doorlopende financiering zonder bouwplan.
6. Huur personeel voor zichtbaar werk en controleer loonwachters en patrouilles.
7. Koop marketing pas wanneer extra bezoekers ook kunnen eten, wachten, rijden en vertrekken.
8. Vergelijk daarna een nieuwe financiële periode. Hogere omzet met nog sneller stijgende kosten is geen verbetering van winst.

Onder **Policies** bestaan prijsvoorstellen van $2, $3 en $5 voor alle geïnstalleerde attracties. Deze controleren de regels vóór uitvoering en behouden reeds afgesproken gastprijzen. Gebruik ze voorzichtig: een familiecarousel en intense coaster hoeven niet hetzelfde optimale tarief te hebben.

### Leningen verstandig gebruiken

Onder **Finance → Loans** leen of betaal je in stappen van $100, binnen de toegestane limiet en scenarioregels. De algemene hoofdsomlimiet is $2.000. Rente is **0,5% per simulatieminuut** over de openstaande hoofdsom: $100 kost dus $0,50 per volle minuut. Dit is geen jaarlijks percentage. Sneller spelen versnelt die kosten.

Rente wordt geleidelijk opgebouwd en op minuutgrenzen geïnd. Onbetaalde rente blijft verschuldigd; zij wordt niet automatisch een nieuwe lening. Voor een aflossing moet ook opgebouwde hele-cent-rente kunnen worden voldaan. Leen voor een concreet bereikbaar inkomstenplan en houd genoeg over voor exploitatie. Geld lenen heropent geldgesloten attracties niet automatisch.

## 16. Personeel, onderhoud, weer en incidenten

### Je team inzetten

Open **Manage → Staff and park life**. Nieuwe parken hebben deze systemen actief. Oudere parken kunnen expliciet **Enable park life** aanbieden. Er passen maximaal twintig teamleden; elke rol kost $1 per dertig simulatieseconden. Controleer de taak en betaalstatus per werknemer.

| Rol | Werk | Controleer bij stilstand |
| --- | --- | --- |
| Mechanic | Onderhoud aan daarvoor gereedstaande attracties | Rit gesloten, leeg, stil en bereikbaar? |
| Handyman | Afval, braaksel en volle bakken aanpakken | Openbaar pad en passend werkgebied? |
| Security | Vandalisme en nood-/distressmeldingen oplossen | Incident binnen bereik en patrouille? |
| Entertainer | Tevredenheid van nabije bezoekers verhogen | Werknemer betaald en bij bezoekers? |
| Service attendant | Bij services helpen en vermoeide/misselijke gasten ondersteunen | Open bereikbare balie of gasten op openbaar pad? |

Selecteer een werknemer, gebruik **Patrol tile** op een openbaar pad om een bestemming aan het werkgebied toe te voegen of eruit te halen. **Whole park** verwijdert die beperking. Een patrouille beperkt werkbestemmingen; reisroutes mogen via het verbonden openbare netwerk lopen. **Find selected** zoekt de werknemer; **Dismiss selected** beëindigt het contract zonder oude loonkosten te wissen.

In **Management → Policies** staat een teamloongrens van $0 tot $20 per betaalvenster van dertig seconden. Te weinig kas of een te laag plafond zet werk stil tot betaling mogelijk is. De grens omvat niet automatisch alle andere parkuitgaven of oude losse medewerkercontracten. **Grounds controls** bevat ook de eerdere losse monteur-/handymanbediening; voorkom dat je onbedoeld dubbel personeel betaalt.

### Weer en schuilplaatsen

De parklevensimulatie kent zonnig, heet, regenachtig en stormachtig weer. Automatisch weer kan elke zestig simulatieseconden veranderen; de weerbeleidknop kan een vaste toestand kiezen. Hitte maakt dorstig. Regen en storm drukken buitentevredenheid. Overdekte bankjes bieden bescherming tijdens rust.

Storm kan blootgestelde attracties sluiten terwijl vertrokken gasten hun rit afronden. Geschikte overdekte attracties kunnen blijven draaien. Na herstel controleert het spel of veilig heropenen mogelijk is. Een eigen sluitverzoek, slecht pad, onderhoud of geldprobleem kan dat nog verhinderen. De decoratieve **Lighting / Weather**-weergave onder Settings is een presentatiekeuze; beheer werkelijk parkweer via **Staff and park life**.

### Incidenten aanpakken

Oranje kaartmarkeringen en de personeelsrapporten wijzen op incidenten. Ernstige vermoeidheid of misselijkheid kan een distressmelding veroorzaken; ongelukkige gasten kunnen meubilair beschadigen. Een beschadigde schuilplaats werkt niet goed totdat zij hersteld is.

Gebruik **Find first incident**, bekijk de behoefte of schade, controleer de route van security en zet zo nodig services of attendants in. Alleen meer beveiliging inhuren verhelpt structureel gebrek aan rust, drank of bereikbare paden niet. Lees bij onderhoud ook de status van de individuele attractie.

## 17. Onderzoek en economische instellingen

Open **Management and economy → Research**. Kies een categorie en financiering: gepauzeerd, $1, $2,50 of $5 per dertig simulatieseconden. Een project vraagt $40 aan daadwerkelijk betaalde investering. Bij beperkt geld kunnen kleinere betaalbare stappen worden verwerkt. Van categorie wisselen bewaart eerdere investering; een voltooide categorie blijft niet onbeperkt doorbetalen.

| Categorie | Beschikbaar bij normaal begin | Via onderzoek |
| --- | --- | --- |
| Gentle | Carousel, Wheel, Cups | Skyline Tower |
| Thrill | Geen | Swing, Drop, Corsair Ship |
| Water | Geen | Timber Falls, River Rapids |
| Transport | Park Railway | Vista Monorail |
| Coasters | Classic Coaster | Steel Launch Coaster, Inverted Coaster |
| Facilities | Food, Drinks, Toilets, Rest, Information | Ice cream, Souvenirs, First aid |

De catalogus toont de werkelijke beschikbaarheid voor jouw park. Een Sandbox met alles beschikbaar of een scenario met eigen regels kan van deze beginset afwijken. Onderzoek begint normaal gepauzeerd zodat je niet ongemerkt je startbudget uitgeeft. Oude parken kunnen eerst **Enable management** nodig hebben; lees die melding omdat voorraad en economische regels daarna veranderen.

**Overview** vat economie en waarschuwingen samen. **Research** bestuurt ontgrendelingen. **Supplies** beheert voorraden. **Marketing** biedt campagnes. **Policies** beheert gezamenlijke prijsacties en personeelsbudget. **History** vergelijkt financiële periodes. **Operations** brengt problemen bij concrete objecten samen. Gebruik die laatste route om van een abstract probleem naar de betrokken attractie, service of gast te gaan.

## 18. Marketingcampagnes

Een marketingcampagne koopt extra bereik; zij garandeert geen klanten of winst. Zorg eerst dat het park open is, aankomsten toestaat, een verbonden ingang heeft en extra bezoekers kan bedienen. Advertenties kosten direct geld. De looptijd is simulatietijd; park sluiten of instroom uitzetten geeft geen refund en pauzeert de campagne niet. Space pauzeert wel de hele simulatie.

| Campagne | Prijs | Looptijd | Effect |
| --- | --- | --- | --- |
| Local flyers | $25 | 5 minuten | Kandidaatinterval verkort van 30 naar 15 seconden |
| Radio ads | $40 | 5 minuten | Kandidaten accepteren maximaal $3 meer toegangsprijs, binnen hun echte portemonnee |
| Family week | $50 | 5 minuten | Thematische instroom, interval van 10 seconden; open rustige attractie vereist |
| Coaster enthusiasts | $50 | 5 minuten | Thematische instroom, interval van 10 seconden; open coaster vereist |

Flyers en radio staan onder **Visitor flow** en via marketingroutes. Van ieder type kan één campagne actief zijn; flyers en radio kunnen samen lopen. Er kan één themapromotie tegelijk lopen. Combineer ze niet op basis van opgetelde beloften: lees het actieve instroominterval en de echte teller. Campagneregels kunnen bepaalde promoties verbieden.

### Een campagne evalueren

1. Bekijk vóór aankoop de kasbuffer, vrije capaciteit en laatste financiële periode.
2. Kies een campagne die past: flyers voor meer kansen op aankomst, radio voor toegangsbereidheid, thema voor bijpassende attracties.
3. Noteer bezoekers, verkoop en kosten. Hervat totdat voldoende campagnetijd is verstreken.
4. Bekijk kandidaten, toegelaten bezoekers en, bij radio, extra toelatingen door het prijsvoordeel. Een kandidaat is nog geen betalende bezoeker.
5. Controleer behoeften en wachtrijen: overbelasting kan nieuwe opbrengsten tenietdoen.
6. Vergelijk na afloop de extra verkoop met campagne- én extra exploitatiekosten. Kies pas daarna een volgende campagne.

Het radio-effect voegt geen geld aan de portemonnee toe. Een gast kan nog steeds weigeren wegens budget of regels. Toegeschreven campagneverkopen zijn een rapportage over het campagnevenster, geen bewijs dat iedere verkoop zonder reclame onmogelijk was.

## 19. De campagne: zes scenario's

De spelcampagne is iets anders dan reclame. Kies **Campaign** in het hoofdmenu. Elk scenario heeft eigen doelen, startregels en ontgrendeling. Je behaalde onderscheiding staat in het spelersprofiel en ontgrendelt het volgende scenario. **Later campaign** laat de vier latere opdrachten kiezen.

| Scenario | Doelen | Deadline | Beginbudget |
| --- | --- | --- | --- |
| Meadow Challenge | 20 nieuwe bezoekers, 5 voltooide bezoekersritten, minstens $750 kas | Geen | $1.000 |
| Boardwalk Sprint | 24 nieuwe bezoekers, 12 voltooide bezoekersritten, minstens $900 kas | 10 minuten | $1.000 |
| Cedar Valley | 30 nieuwe bezoekers en tevredenheid 65 | 20 minuten | $2.000 |
| Research Ridge | 1 gefinancierd onderzoeksproject afronden en 20 bezoekersritten | 20 minuten | $2.000 |
| Lakeside Services | 3 services plaatsen, 12 klanten bedienen en 30 bezoekers | 20 minuten | $2.000 |
| Grand Festival | 60 bezoekers, $150 verkoop en tevredenheid 65 | 20 minuten | $2.500 |

Lees in **Scenarios and goals** de voortgang en status. Een “rit” in een bezoekersdoel is een voltooide gastreis, geen nieuwe baan of lege testronde. Nieuwe aankomsten tellen vanaf het scenariobegin. Bestaand verkeer is niet automatisch een voorsprong. Een kasdoel is een saldo-eis en verschilt van een winstdoel.

### Meadow en Boardwalk aanpakken

**Meadow** vereist gratis parktoegang. Ritten mogen geld kosten. Radio en themapromoties zijn niet beschikbaar; flyers kunnen wel. Onderzoek is begrensd op $2,50 per dertig seconden. Maak eerst een bereikbare open attractie, houd exploitatie klein en laat genoeg nieuwe bezoekers daadwerkelijk een rit afmaken. Bewaak de $750 kas terwijl je bouwt.

**Boardwalk** vereist gratis ritten. Denk dus aan toegangsinkomsten en passende serviceprijzen. Bouw niet blind extra coasters die niets aan het ritprijsdoel kunnen verdienen. Pauzeer om te plannen, maar besef dat na hervatten ook de tienminutenteller loopt. Bewaak tegelijk instroom, ritvoltooiingen en de benodigde kas.

### De vier latere opdrachten aanpakken

Bij **Cedar Valley** zijn bereikbare behoeften en rustige routes belangrijk om tevredenheid op peil te houden. **Research Ridge** vraagt betaald onderzoek: alleen een al beschikbaar model neerzetten rondt geen nieuw gefinancierd project af. Kies tijdig categorie en budget. **Lakeside Services** vraagt niet alleen gebouwen maar echte bedieningen; lege voorraad of verkeerde oriëntatie blokkeert voortgang. **Grand Festival** combineert capaciteit, omzet en tevredenheid: controleer alle drie in plaats van uitsluitend bezoekers aan te trekken.

Een mislukte deadline blijft bij dat park opgeslagen. Laad een eerdere eigen save of start een nieuwe poging. Eerder verdiende profielonderscheidingen blijven behouden. Probeer geen scenario te veranderen in een bestaande lopende uitdaging; start een aparte poging zodat doelen en regels helder blijven.

## 20. Eigen scenario's en de editor

### Een opdracht ontwerpen

1. Begin in een Sandbox met het gewenste landschap en de gewenste startgebouwen.
2. Open **Scenarios and editor** en kies **New from this park**. De editor bewaart je oorspronkelijke park apart.
3. Gebruik **Build park** voor terrein, paden, decoratie, attracties en services. De banner maakt duidelijk dat je de startopstelling bewerkt.
4. Keer terug naar de editor en geef naam en beschrijving. Beschrijf voor de speler ook bijzondere beperkingen.
5. Stel op **Goals and rules** maximaal vier verschillende doelen in, met maatstaf, doelwaarde en eventueel deadline.
6. Stel startbudget, toegangsbeleid, onderzoek, leningen en instroom in. Op **Available content** kies je bouwbare modellen en services.
7. Kies **Save scenario**, daarna **Test scenario** om de opdracht echt uit te proberen.
8. Gebruik **Return to editor** om je ontwerp terug te krijgen en te verbeteren. Testen verdient geen officiële campagneonderscheidingen.
9. Met **Return to park** en bevestiging verlaat je de editor en keert het oorspronkelijke park terug. Opgeslagen scenario's blijven bestaan.

Doelen kunnen onder andere aankomsten, ritten, klanten, verkoop, bedrijfsresultaat, kas, tevredenheid, services, parkwaarde en betaald onderzoek gebruiken. Stem deadlines af op looptijd, loopafstanden en werkelijk betaalbare bouw. Beperkingen op nieuwe bouw verwijderen reeds in de template aanwezige gebouwen niet.

### Budget en doelwaarden eerlijk instellen

Het startbudget is de financiële toewijzing, terwijl eerdere bouwuitgaven van de startopstelling bij het park horen. Een budget onder de al geboekte besteding wordt geweigerd. Verkoopdoelen tellen nieuwe scenarioverkopen; een bedrijfsresultaatdoel kan de bestaande bedrijfshistorie van de template meenemen. Begin voor een heldere opdracht met een bewuste, geteste opstelling en lees de getoonde voortgang.

### Scenario's delen

De bibliotheek zoekt op naam, toont omschrijving, kaartgrootte, doelen en compatibiliteit. **Play selected** begint na bevestiging een poging. **Edit selected** opent een bewerkbare kopie. Hernoemen en archiveren vragen bevestiging; archiveren verplaatst het bestand naar het scenarioarchief.

**Export** maakt een `.parkscenario.json`. **Import** controleert het gekozen bestand voordat het aan de bibliotheek wordt toegevoegd. Een scenario bevat de startopstelling en regels; het is geen Windows-programma en ook geen campagneprofiel. Een geïmporteerde eigen opdracht kent geen officiële campagneonderscheidingen toe. Niet-ondersteunde of beschadigde bestanden tonen een fout; bewaar het origineel wanneer je een probleem onderzoekt.

## 21. Tips: drie keer per park

Het **lampicoon** naast Finance en Rides geeft **twee adviezen** over de actuele toestand, bijvoorbeeld bereikbaarheid, behoeften, ontevreden gasten, cash, verlies, prijsafwijzingen of rente. Het advies past niets automatisch aan en belooft geen winst. Lees beide tips en controleer daarna de genoemde cijfers.

Je kunt de knop **drie keer per park** gebruiken. Niet drie keer per keer opstarten. Opslaan, herladen, een vorige backup openen of een bouwbewerking terugdraaien herstelt het aantal niet. **Save As New Park** neemt verbruikte tips mee. Alleen een werkelijk nieuwe spelstart heeft een nieuwe limiet.

Bij het eerste advies in een nog niet opgeslagen park wordt een bibliotheekplek gemaakt om het verbruik vast te leggen. Dat vervangt normaal opslaan van latere voortgang niet: blijf F5 gebruiken. Als die registratie niet kan worden geschreven, krijg je een fout en geen nieuw advies. De adviezen werken lokaal; er is geen internetdienst nodig.

## 22. Opslaan, backups en een andere pc

### Een park veilig bewaren

**F5** of **Menu → Save** schrijft het huidige park naar zijn bibliotheekplek. Een nieuwe spelstart krijgt bij de eerste save een eigen plek. **Save As New Park** maakt een aparte kopie. **Rename** verandert de zichtbare naam van de bibliotheekplek. **Delete** archiveert een park; het verwijdert het niet meteen definitief van de schijf.

**Load Park** toont je opgeslagen parken. **Continue latest park** kiest de nieuwste save. **F9** laadt het huidige opgeslagen park; **Shift+F9** vraagt de vorige geldige backup. Sla vóór zo'n herstel eventueel je huidige toestand apart op. Een backup is de vorige save, geen volledige tijdlijn. Lees bevestigingen bij het verlaten of vervangen van onopgeslagen werk.

Je park bewaart onder andere terrein, bouw, bezoekers, voertuigen, prijzen, voorraad, financiën, personeel, onderzoek, lopende campagnes, scenariovoortgang en Tips-verbruik. Tijdelijke grafieken, selectie en undo-geschiedenis hoeven niet allemaal mee te gaan. De huidige parkindeling is saveversie 52; een oudere executable kan een nieuwere save niet noodzakelijk lezen. Bewaar bij upgrades een kopie van je spelersmap.

### Wat staat waar?

| Bestand of map | Functie |
| --- | --- |
| `%LOCALAPPDATA%\Parkbound\saves` | Spelersmap: parken, voorkeuren, profiel en bibliotheken |
| `parks` binnen de spelersmap | Afzonderlijke opgeslagen parkplekken met metadata en backups |
| `deleted-parks` | Gearchiveerde parkplekken |
| `campaign.json` en backup | Verdiende campagneonderscheidingen |
| `display-settings.json`, `sound-settings.json`, `accessibility-settings.json` | Persoonlijke instellingen |
| `docs/HANDLEIDING_NL.html` naast de executable | Deze offline spelershandleiding |

Sluit Parkbound voordat je bestanden kopieert. Neem voor een andere pc de **hele spelersmap** mee, niet alleen één parkbestand. Installeer daar dezelfde of een compatibele nieuwere Parkbound en zet de map terug. Start daarna, laad je park en controleer de instellingen voor het nieuwe scherm. De broncode op OneDrive synchroniseert AppData niet vanzelf.

De ontwikkelcheckout staat op `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`. Dat is de code en technische documentatie, niet automatisch je spelersprofiel. Bouwers lezen **RESUME_DEVELOPMENT.md** om ontwikkeling te hervatten. Google Drive-synchronisatie is uitgeschakeld.

## 23. Problemen oplossen

### Er komen geen bezoekers binnen

1. Hervat de simulatie; gepauzeerd komen geen nieuwe kandidaten.
2. Open **Visitor flow** en controleer park open, instroom aan en een ingestelde ingang.
3. Controleer dat die ingang op een openbaar pad ligt dat het parknetwerk bereikt.
4. Bekijk kandidaten en weigeringen. Een te hoge toegangsprijs kan betaalbare ritten alsnog onbereikbaar maken.
5. Controleer de actieve bezoekerslimiet en geef bestaande gasten tijd om te vertrekken.
6. Koop pas daarna reclame. Marketing repareert geen ontbrekende ingang.

### Een attractie wil niet open of ontvangt niemand

1. Rechtsklik de juiste attractie en lees de concrete Operations-melding.
2. Controleer Access: correcte eigen ingang, wachtrij met openbare aansluiting en afzonderlijk bereikbaar uitgangspad.
3. Controleer alle randhoogten, vooral na grondaanpassingen of hellingbouw.
4. Controleer baanlus, vereiste test, trein-/blokruimte, stilstand en leegte voor configuratiewijzigingen.
5. Los onderhoud, storm of geldsluiting op. Open daarna expliciet opnieuw als nodig.
6. Is de rit open maar leeg? Bekijk prijs, gastvoorkeuren, beschikbare wallets en minimumlading. Volg een gast via Find.

### Bezoekers staan vast of de wachtrij loopt niet door

Controleer eerst het openbare pad terug naar de ingang en de uitgang van de rit. Een wachtrij van attractie A helpt attractie B niet. Zoek de eerste niet-aansluitende rand, te hoge/lage tegel of verkeerde poortoriëntatie. Roteer de kaart om verborgen onderdelen te zien. Gebruik Underground view voor begraven routes. Repareer de verbinding en laat even spelen voordat je nóg meer paden toevoegt.

### Een service verkoopt niets

Controleer opening, voorraad, geld voor een hele bestelling en een openbaar pad aan de bedieningskant. Daarna prijs en echte gastbehoefte. Een souvenirwinkel helpt een dorstige gast niet. Een bezoeker kan nog onderweg of in bediening zijn; verkoop wordt bij afronding zichtbaar. Gebruik Supplies en de individuele Services-selectie om niet per ongeluk het verkeerde gebouw te wijzigen.

### Het park verliest geld terwijl het druk is

Pauzeer en bekijk **Finance** plus **Management → History**. Splits bouw, voorraadinkoop, lonen, onderhoud, advertenties, onderzoek en rente. Stop onnodige doorlopende uitgaven, maak onbereikbare capaciteit bruikbaar en pas prijzen gecontroleerd aan. Een lening geeft ademruimte maar verhoogt rente; zij maakt een structureel verliesgevend park niet vanzelf rendabel.

### Onderzoek of personeel lijkt niets te doen

Controleer of financiering gepauzeerd is, de categorie al klaar is, cash ontbreekt of scenarioregels de instelling beperken. Controleer bij personeel loonstatus, loonplafond, openbaar pad, patrouille en daadwerkelijk geschikt werk. Een monteur wacht terecht op een lege en gestopte attractie. Een medewerker op een los padeiland kan niet naar het probleem lopen.

### Knoppen, hoogtes of vensters reageren onverwacht

Lees de hovertekst en sluit actieve tekstinvoer. Hetzelfde PgUp/PgDn bedient afhankelijk van het venster een lijstpagina of bouwhoogte. C draait soms een gebouw en wisselt soms een padvorm. Tijdens coasterbouw zet handmatig hoogte wijzigen Follow heights uit. Een sleepbeweging met rechts is geen contextklik. Druk Escape om naar inspectie terug te keren en probeer dan opnieuw.

Bij te kleine tekst: interfaceschaal aanpassen. Bij verborgen bediening: boven het venster scrollen. Bij verkeerd scherm: F11. Bij geen geluid: Mute, volumes, Windows-uitvoer en Reconnect audio device controleren. Bij een onleesbare save: probeer de getoonde backup en bewaar het beschadigde bestand voor onderzoek. Laat herstelmeldingen leidend zijn; overschrijf niet blind je enige kopie.

## 24. Sneltoetsen en context

Dit zijn de standaardtoetsen. **Controls & accessibility** toont eventuele eigen toewijzingen. In een invoerveld typ je tekst; in een lijst kunnen pijltjes selecteren; in Explore kiezen cijfers groepen. Een toets kan daarom bewust een andere functie hebben dan op de vrije kaart.

| Toets of gebaar | Functie |
| --- | --- |
| Korte rechterklik | Instellingen van aangeklikte attractie/service |
| Rechts slepen / midden slepen | Camera verplaatsen |
| WASD / pijltjes | Camera verplaatsen buiten speciale invoercontext |
| Wiel / + / − boven kaart | Zoomen |
| Wiel boven venster | Door vensterinhoud scrollen |
| Q / E | Kaart roteren |
| Home | Kaart passend in beeld |
| Space | Pauze/hervatten |
| 1 / 2 / 3 | 1× / 2× / 4×, buiten Explore en invoer |
| F2 | Explore; daarin 1–5 voor de groepen |
| L / M / N / Y | Guests / Services / Finance / Rides |
| Z | Coastereditor |
| Ctrl+D | Ontwerpbibliotheek |
| P / J | Openbaar pad / wachtrij |
| I / K | Attractie-ingang / -uitgang |
| X / Delete | Sloopgereedschap / geselecteerd onderdeel verwijderen |
| B / V | Terrein omhoog / omlaag |
| C | Padvorm of oriëntatie; coasterrichting in editor |
| PgUp / PgDn | Bouwhoogte; pagina/keuze in daarvoor bestemd venster |
| Enter in coastereditor | Volgend stuk toevoegen |
| G / U | Raster / ondergrondweergave |
| O / F | Geselecteerde rit of service openen/sluiten / rit vinden, in passende context |
| F6 / F7 | Prijs − / + $0,50 voor de actieve selectie |
| Ctrl+Z / Ctrl+Y | Undo / redo van gepauzeerde bouw of de actieve draft |
| F5 / F9 / Shift+F9 | Opslaan / laden / vorige backup |
| H / F1 | Help |
| F10 | Geluid |
| F11 | Veilige vensterweergave herstellen |
| Tab / Shift+Tab / Enter | Menufocus vooruit / terug / activeren |
| Escape | Actief venster of gereedschap sluiten; daarna parkmenu |

**T** en **R** zijn oude terrein-/demonstratieherstelacties met specifieke voorwaarden, geen normale route om een bewoond park veilig te verbouwen. Gebruik bij gewoon spelen de expliciete bouwgereedschappen en Home voor de camera. Sla op voordat je een herstelfunctie gebruikt.

## 25. Begrippen en verder lezen

| Begrip | Betekenis |
| --- | --- |
| Attractie / Ride | Eén zelfstandig beheerd object met eigen toegang, bedrijfstoestand en boekhouding |
| Facility / Service | Voorziening die een behoefte bedient en voorraad kan verbruiken |
| Public path | Openbaar deel van het loopnetwerk |
| Queue | Aan één attractie gekoppeld wachtpad |
| Randhoogte | Hoogte waarop een pad aan de buur aansluit |
| Draft | Bewerkbaar baanontwerp vóór Apply |
| Block | Beveiligd baandeel dat ruimte reserveert voor een trein |
| Candidate | Bezoeker die overweegt binnen te komen; nog niet noodzakelijk toegelaten |
| Admission | Een daadwerkelijke toelating tot het park |
| Revenue / Sales | Verkoopopbrengst, vóór aftrek van kosten |
| Upkeep | Terugkerende exploitatie-/onderhoudskosten |
| Payroll cap | Uitgavenplafond voor teamlonen per betaalvenster |
| Cashflow | Geld dat werkelijk binnenkomt of vertrekt |
| Operating result | Bedrijfsopbrengst minus geboekte bedrijfskosten |
| Scenario | Parkstart met opdracht en vaste regels |
| Campaign | Reeks officiële scenario's; in marketingcontext een advertentieactie |
| Backup | Vorige geldige opslagversie, geen onbeperkte geschiedenis |

Voor spelen: begin met [hoofdstuk 3](#3-je-eerste-speelbare-park-stap-voor-stap), gebruik [de menuwegwijzer](#5-schermindeling-en-menus) en zoek bij vastlopen in [problemen oplossen](#23-problemen-oplossen). Voor bouwen zijn [paden](#8-paden-wachtrijen-poorten-en-hoogte) en [coasters](#11-coasters-bouwen-van-station-tot-eerste-rit) de belangrijkste referenties. Voor beheer begin je bij [financiën](#15-financien-en-winst), [personeel](#16-personeel-onderhoud-weer-en-incidenten) en [onderzoek](#17-onderzoek-en-economische-instellingen).

Deze handleiding wordt samen met broncode en Windows-pakket bijgewerkt. De technische hervatgids, versiegeschiedenis en sessieverslagen leggen ontwikkelwerk en teststatus vast. Ze zijn geen extra speldoelen. De ontwikkel-pc wordt uitsluitend gebruikt voor functionele controles, niet als prestatiereferentie voor het product.
