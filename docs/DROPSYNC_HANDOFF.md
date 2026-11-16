# DropSync-overdracht — Parkbound 0.94.0

Bestemming: `Rust builds/Parkbound` in de bestaande persoonlijke DropSync-bibliotheek van michiel. De reeds aanwezige map heet `Rust builds`; die spelling blijft behouden. Lokale gesynchroniseerde bestemming: `C:\Users\Michiel Strijbos\Downloads\DropSync\Rust builds\Parkbound`.

## Inhoud

- `Parkbound-0.94.0-codebase.zip`: volledige versiebeheer-broncode, assets, licenties, scripts en documentatie van de vastgelegde Git-commit. Geen actieve .git-map, target, persoonlijke saves of authenticatiebestanden.
- `Parkbound-0.94.0-windows-x64.zip`: zelfstandig speelbare Windows-build. Uitpakken, `Parkbound.exe` openen en **Play Sunhaven Gardens** kiezen.
- `GESPREK.md`: bestaande beschikbare gesprekstekst, aangevuld met beschikbare gebruiker-/assistentberichten van deze ontwikkelbeurt. Dekking is gedeeltelijk; geen verborgen redeneringen, tools of media en geen native hervatbare chat.
- `START_HIER.md` en `VOORTGANG.md`: hervatten, controles, beperkingen en volgende stappen.
- `SHA256SUMS.txt`: hashes van de overdrachtsbestanden. De lokale ontvangstregistratie in `artifacts/dropsync-transfer/` legt de waargenomen cloudstatus apart vast.

## Hervatten

De actuele bron staat op https://github.com/strijbosmh-afk/RailRust-Tycoon, branch `codex/landscape-graphics`, PR 86. De exacte broncommit wordt ook in `START_HIER.md` vermeld. Clone bij voorkeur GitHub en kies die branch; de bron-ZIP is een volledige offline bronkopie zonder Git-historie. Lees daarna `RESUME_DEVELOPMENT.md`, `STATUS.md`, `INSTALLATIE.md` en `docs/LANDSCAPE_GRAPHICS_VERIFICATION.md`.

De oorspronkelijke gedeelde werkmap blijft `C:\Users\Michiel Strijbos\OneDrive - RZ Heilig Hart Tienen vzw\Documenten\ChatGPT\Railroad Tycoon`. Rust 1.93.1, Git, Microsoft C++ Build Tools en Windows SDK zijn nodig voor bouwen. De eerste bronbuild downloadt de vastgelegde dependencies; de Windows-build speelt offline.

De overdracht is een versiegebonden kopie. Een lokaal gekopieerd bestand is pas als cloudopslag bevestigd nadat DropSync de upload heeft afgerond en de nieuwe cloudinhoud is teruggelezen. De afrondingsmelding en lokale ontvangstregistratie vermelden dat resultaat; dit document zelf is geen cloudbewijs. Google Drive blijft een afzonderlijk archief met eigen verificatie en openstaande wachtrij.

## Vervolgwerk

De landschapsverbetering wijzigt geen simulatie of saveformaat (51/51). Onafhankelijke spelersacceptatie, een representatieve Windows-pc en de volledige hardware-/beeldschermmatrix blijven open. Gebruik PC1830 niet als prestatiereferentie.
