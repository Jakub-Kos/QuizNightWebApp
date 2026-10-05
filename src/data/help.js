// Help page content. Text in `backticks` is shown as code. The sk block is intentionally Hungarian,
// like the rest of the sk translations.
export const HELP = {
  en: {
    title: "How to make a quiz",
    templates: {
      title: "Templates",
      questions: "Questions template (CSV)",
      scores: "Scores template (CSV)",
      teams: "Teams template (CSV)",
      google: "Copy the Google Sheets template",
      importHint: "In Google Sheets: File → Import → Upload, then pick the CSV.",
    },
    sections: [
      {
        id: "start", title: "Getting started", ordered: true,
        items: [
          "Write the questions in Google Sheets (or any spreadsheet) with the columns from the template.",
          "In the library click New quiz, paste the Sheet link (or upload the CSV) and drop in the images.",
          "Link a scores Sheet (or plan to type the points into the score table during the quiz), then add the teams: with a Sheet, Load names from live scores fills them in.",
          "Open Check: fix everything red and look through the question thumbnails.",
          "Start the show on the TV and press `Shift+P` on your laptop to open the presenter window.",
        ],
        note: "Quizzes are saved only in this browser. Use Export (.zip) as a backup or to move a quiz to another computer.",
      },
      {
        id: "questions", title: "The questions sheet",
        intro: "One row per question. The first row is the header and is skipped. Columns are read by their position, so keep the order.",
        table: {
          head: ["Column", "What to put there"],
          rows: [
            ["A `Kolo` / `Round`", "On the first question of each round: `Round 1 - Round title` (or `Kolo 1 - …`). Empty on the other rows."],
            ["B `Q#`", "Question number, shown big on screen. Every question row needs a number."],
            ["C `Otázka`", "The question text."],
            ["D `Typ Odpovědi`", "The question type (see below). Empty means Written."],
            ["E–H `A B C D`", "The four options, only for ABCD."],
            ["I `Správná odpověď`", "The answer shown at the reveal."],
            ["J `Zdroj`", "File name of the image, audio or video, e.g. `paris.jpg`. It must match an uploaded file."],
          ],
        },
        note: "Rounds whose title contains TEST, or rounds named `Kolo X`, are hidden in the show; use them to try things out.",
      },
      {
        id: "types", title: "Question types",
        table: {
          head: ["Type", "On screen", "Fill in"],
          rows: [
            ["`Written`", "The question, then the answer.", "Answer."],
            ["`Numeric`", "Like Written, asks for a number.", "A number as the answer."],
            ["`ABCD`", "Four options; the correct one lights up.", "Options A–D, answer `A`, `B`, `C` or `D`."],
            ["`Yes/No`", "Yes and No tiles; the correct one lights up.", "Answer `Yes` or `No` (in English)."],
            ["`Image`", "The image under the question (click it or press Z to enlarge).", "File in Zdroj."],
            ["`PImage`", "The image full screen first, then the question.", "File in Zdroj."],
            ["`Audio`", "A player under the question.", "File in Zdroj."],
            ["`Video`", "A video player under the question.", "File in Zdroj."],
            ["`PVideo`", "The video full screen first (starts by itself), then the question.", "File in Zdroj."],
            ["`Top5`", "Five hidden answers, revealed together.", "First answer on the question row; the other four on the next rows with only a number in Q# and the answer."],
            ["`Sort`", "2–10 items shown mixed up with letters A, B, C…; the reveal puts them in order and shows the letters, e.g. `C A D B`. With pictures in Zdroj the items are picture cards.", "The items in the correct order, the first on the question row, the others on the next rows with only a number in Q#. Each row: the item in column `A`, the letter it is shown under in the answer column (A, B, C… each once, mixed up), optionally a picture in Zdroj. The answer column then reads the answer top to bottom, e.g. `B D A C`, ready for marking. Without letters the show mixes the items itself and Check tells you which letters it picked."],
          ],
        },
        examples: {
          "Written": {
            "text": "What is the capital of France?",
            "answer": "Paris"
          },
          "Numeric": {
            "text": "How many legs does a spider have?",
            "answer": "8"
          },
          "ABCD": {
            "text": "Which planet is the largest?",
            "options": {
              "A": "Mars",
              "B": "Jupiter",
              "C": "Venus",
              "D": "Mercury"
            },
            "answer": "B"
          },
          "Yes/No": {
            "text": "Is a whale a mammal?",
            "answer": "Yes"
          },
          "Image": {
            "text": "Where and when was this photo taken?",
            "answer": "Berlin, 1989"
          },
          "PImage": {
            "text": "In which city did this happen?",
            "answer": "Paris"
          },
          "Audio": {
            "text": "Name this jingle",
            "answer": "Quiz Night jingle"
          },
          "Video": {
            "text": "What is shown in this clip?",
            "answer": "A test pattern"
          },
          "PVideo": {
            "text": "How many seconds did the clip last?",
            "answer": "5"
          },
          "Sort": {
            "text": "Put these inventions in order, oldest first",
            "items": ["Printing press", "Steam engine", "Telephone", "Aeroplane", "Internet"],
            "letters": ["C", "A", "E", "B", "D"]
          },
          "Top5": {
            "text": "Name the 5 largest countries",
            "answer": [
              "Russia",
              "Canada",
              "China",
              "USA",
              "Brazil"
            ]
          }
        },
        mediaStep: "Media first", previewHint: "Each type shows the rows to write into the questions sheet and what they look like on screen. Click a preview to enlarge it.", sheetLabel: "In the sheet",
      },
      {
        id: "media", title: "Images and media",
        items: [
          "File names in Zdroj must match the uploaded files. Capital letters and folders do not matter.",
          "Images: jpg, png, webp, gif, avif. Audio: mp3, m4a, wav, ogg. Video: mp4 (H.264) or webm; iPhone .mov files may not play in every browser.",
          "Keep images under about 2 MB; large files make the quiz and its export slow.",
          "Drag a whole folder onto the media area of the setup page. Check lists every missing file.",
          "Media can also be a web link (`https://…`) to a picture hosted elsewhere, but then it needs internet during the show.",
        ],
      },
      {
        id: "google", title: "Using Google Sheets",
        items: [
          "Make a copy of the template (button above), or import the questions template CSV into a new sheet.",
          "Share → General access → Anyone with the link (Viewer). Without it the app cannot read the sheet.",
          "Copy the link from the browser address bar while the right tab is open: the `#gid=…` part tells the app which tab to read. A link without it reads the first tab. A Publish to web CSV link works too.",
          "The questions are loaded again every time the show starts, so typos can be fixed until the last minute (then reload the show).",
          "The same spreadsheet can have a second tab for the scores (see Scores) — paste that tab's link into the scores field.",
        ],
      },
      {
        id: "teams", title: "Teams",
        items: [
          "If the quiz has a live scores Sheet, Load names from live scores adds every team from it. Add teams by hand, or import a CSV with a `Název týmu` (or `Team name`) column and optional `Počet hráčů` / `Players` and `Motto` / `Quote` columns. A Google Form registration export works as it is.",
          "Photos: click the square next to a team to upload one. Google Drive links from a form cannot be shown.",
          "Past results: a CSV with the team name and one column per past quiz whose name contains `Rank`, plus an optional `Accuracy` column. They appear on the welcome screen.",
          "Team names must match the names in the scores, otherwise photos and colors are missing on the leaderboard.",
          "Photos, mottos and past results are optional. When fewer than half of the teams have a photo or motto, the welcome screen shows all teams at once as a team wall instead of one card at a time; teams without a photo get their initials. Settings in the show can force either layout.",
        ],
      },
      {
        id: "scores", title: "Scores",
        items: [
          "Option 1, Google Sheet: a tab with a `Team Name` (or `Název týmu`) column and one column per round named `Round 1`, `Round 2`, … (or `Kolo 1`, …). Points like `7,5` or `7.5` both work. It is reloaded every 10 seconds; any column whose name contains “round” or “kolo” counts as a round, other columns (like a total) are ignored.",
          "Option 2, score table: leave the Sheet link empty and type the points into the score table (Scores button in the presenter window). Enter jumps to the next team.",
          "The leaderboard takes the latest round with points and reveals the new standings step by step.",
        ],
      },
      {
        id: "show", title: "Running the show",
        items: [
          "Drag the window to the TV and press F11. Calibrate each new screen once with `Shift+C`.",
          "After the welcome screen come the rules and the prizes. Write your own rules and what each place wins on the setup page (Rules and prizes), or turn either screen off.",
          "On your laptop press `Shift+P`: the presenter window controls the TV and shows the answers.",
          "Before the start, open Attendance (button in the presenter window): tick the teams that came, correct the players and place each team on the seating map (180° or 270° around you). During the leaderboard reveal the presenter window then shows which way to face each team.",
          "Every round has two buttons: Questions (questions only, with a timer, for the writing part) and Answers (goes through again and reveals them).",
          "Keys: `→` next step, `←` back, `Z` enlarge the image, `Esc` back to the dashboard, `Shift+D` dashboard, `Shift+C` calibration, `Shift+P` presenter.",
        ],
      },
    ],
  },

  cs: {
    title: "Jak připravit kvíz",
    templates: {
      title: "Šablony",
      questions: "Šablona otázek (CSV)",
      scores: "Šablona bodů (CSV)",
      teams: "Šablona týmů (CSV)",
      google: "Zkopírovat šablonu v Google Tabulkách",
      importHint: "V Google Tabulkách: Soubor → Importovat → Nahrát a vyberte CSV.",
    },
    sections: [
      {
        id: "start", title: "Začínáme", ordered: true,
        items: [
          "Napište otázky v Google Tabulkách (nebo jiné tabulce) se sloupci podle šablony.",
          "V knihovně klikněte na Nový kvíz, vložte odkaz na tabulku (nebo nahrajte CSV) a přetáhněte obrázky.",
          "Připojte tabulku s body (nebo body zapisujte během kvízu do tabulky bodů) a pak přidejte týmy: s tabulkou je doplní tlačítko Načíst názvy ze živého skóre.",
          "Otevřete Kontrolu: opravte vše červené a projděte si náhledy otázek.",
          "Spusťte show na televizi a na notebooku stiskněte `Shift+P` pro okno moderátora.",
        ],
        note: "Kvízy se ukládají jen v tomto prohlížeči. Export (.zip) slouží jako záloha nebo k přenesení na jiný počítač.",
      },
      {
        id: "questions", title: "Tabulka otázek",
        intro: "Jeden řádek = jedna otázka. První řádek je hlavička a přeskakuje se. Sloupce se čtou podle pořadí, takže ho neměňte.",
        table: {
          head: ["Sloupec", "Co do něj patří"],
          rows: [
            ["A `Kolo` / `Round`", "U první otázky každého kola: `Kolo 1 - Název kola` (nebo `Round 1 - …`). Na ostatních řádcích prázdné."],
            ["B `Q#`", "Číslo otázky, zobrazí se velké na obrazovce. Každý řádek s otázkou potřebuje číslo."],
            ["C `Otázka`", "Text otázky."],
            ["D `Typ Odpovědi`", "Typ otázky (viz níže). Prázdné znamená Written."],
            ["E–H `A B C D`", "Čtyři možnosti, jen pro ABCD."],
            ["I `Správná odpověď`", "Odpověď, která se ukáže při odhalení."],
            ["J `Zdroj`", "Název souboru s obrázkem, zvukem nebo videem, např. `pariz.jpg`. Musí odpovídat nahranému souboru."],
          ],
        },
        note: "Kola, jejichž název obsahuje TEST, nebo kola pojmenovaná `Kolo X` se v show nezobrazí; hodí se na zkoušení.",
      },
      {
        id: "types", title: "Typy otázek",
        table: {
          head: ["Typ", "Na obrazovce", "Vyplňte"],
          rows: [
            ["`Written`", "Otázka, pak odpověď.", "Odpověď."],
            ["`Numeric`", "Jako Written, ptá se na číslo.", "Číslo jako odpověď."],
            ["`ABCD`", "Čtyři možnosti; správná se rozsvítí.", "Možnosti A–D, odpověď `A`, `B`, `C` nebo `D`."],
            ["`Yes/No`", "Dlaždice Yes a No; správná se rozsvítí.", "Odpověď `Yes` nebo `No` (anglicky)."],
            ["`Image`", "Obrázek pod otázkou (kliknutím nebo klávesou Z se zvětší).", "Soubor ve sloupci Zdroj."],
            ["`PImage`", "Nejdřív obrázek přes celou obrazovku, pak otázka.", "Soubor ve sloupci Zdroj."],
            ["`Audio`", "Přehrávač pod otázkou.", "Soubor ve sloupci Zdroj."],
            ["`Video`", "Videopřehrávač pod otázkou.", "Soubor ve sloupci Zdroj."],
            ["`PVideo`", "Nejdřív video přes celou obrazovku (spustí se samo), pak otázka.", "Soubor ve sloupci Zdroj."],
            ["`Top5`", "Pět skrytých odpovědí, odhalí se najednou.", "První odpověď na řádku otázky; zbylé čtyři na dalších řádcích jen s číslem v Q# a odpovědí."],
            ["`Sort`", "2–10 položek zamíchaných a označených písmeny A, B, C…; při odhalení se seřadí a ukáže se pořadí písmen, např. `C A D B`. S obrázky ve sloupci Zdroj jsou položky obrázkové karty.", "Položky ve správném pořadí, první na řádku otázky, další na následujících řádcích jen s číslem v Q#. Na každém řádku: položka ve sloupci `A`, písmeno, pod kterým se zobrazí, ve sloupci odpovědi (A, B, C… každé jednou, zamíchaně), případně obrázek ve Zdroji. Sloupec odpovědi pak shora dolů ukazuje odpověď, např. `B D A C`, připravenou na opravování. Bez písmen show položky zamíchá sama a Kontrola vypíše, která písmena zvolila."],
          ],
        },
        examples: {
          "Written": {
            "text": "Jaké je hlavní město Francie?",
            "answer": "Paříž"
          },
          "Numeric": {
            "text": "Kolik nohou má pavouk?",
            "answer": "8"
          },
          "ABCD": {
            "text": "Která planeta je největší?",
            "options": {
              "A": "Mars",
              "B": "Jupiter",
              "C": "Venuše",
              "D": "Merkur"
            },
            "answer": "B"
          },
          "Yes/No": {
            "text": "Je velryba savec?",
            "answer": "Yes"
          },
          "Image": {
            "text": "Kde a kdy byla fotka pořízena?",
            "answer": "Berlín, 1989"
          },
          "PImage": {
            "text": "Ve kterém městě se to stalo?",
            "answer": "Paříž"
          },
          "Audio": {
            "text": "Co je to za znělku?",
            "answer": "Znělka Quiz Night"
          },
          "Video": {
            "text": "Co je na videu?",
            "answer": "Testovací obrazec"
          },
          "PVideo": {
            "text": "Kolik sekund trvalo video?",
            "answer": "5"
          },
          "Sort": {
            "text": "Seřaďte vynálezy od nejstaršího",
            "items": ["Knihtisk", "Parní stroj", "Telefon", "Letadlo", "Internet"],
            "letters": ["C", "A", "E", "B", "D"]
          },
          "Top5": {
            "text": "Vyjmenujte 5 největších zemí světa",
            "answer": [
              "Rusko",
              "Kanada",
              "Čína",
              "USA",
              "Brazílie"
            ]
          }
        },
        mediaStep: "Nejdřív médium", previewHint: "U každého typu jsou řádky, které napíšete do tabulky otázek, a jak pak vypadají na obrazovce. Kliknutím na náhled ho zvětšíte.", sheetLabel: "V tabulce",
      },
      {
        id: "media", title: "Obrázky a média",
        items: [
          "Názvy souborů ve sloupci Zdroj musí odpovídat nahraným souborům. Na velikosti písmen a složkách nezáleží.",
          "Obrázky: jpg, png, webp, gif, avif. Zvuk: mp3, m4a, wav, ogg. Video: mp4 (H.264) nebo webm; soubory .mov z iPhonu nemusí jít přehrát ve všech prohlížečích.",
          "Obrázky držte zhruba pod 2 MB; velké soubory zpomalují kvíz i jeho export.",
          "Na média v nastavení kvízu můžete přetáhnout celou složku. Kontrola vypíše všechny chybějící soubory.",
          "Médium může být i webový odkaz (`https://…`) na obrázek jinde na internetu, pak ale během show potřebujete připojení.",
        ],
      },
      {
        id: "google", title: "Google Tabulky",
        items: [
          "Zkopírujte si šablonu (tlačítko výše), nebo importujte šablonu otázek v CSV do nové tabulky.",
          "Sdílet → Obecný přístup → Kdokoli s odkazem (čtenář). Bez toho aplikace tabulku nepřečte.",
          "Odkaz zkopírujte z adresního řádku, když máte otevřený správný list: část `#gid=…` říká, který list se má číst. Odkaz bez ní čte první list. Funguje i CSV odkaz z Publikovat na webu.",
          "Otázky se načítají znovu při každém spuštění show, takže překlepy jde opravit do poslední chvíle (pak show obnovte).",
          "Stejný soubor může mít druhý list s body (viz Body) — do pole pro body vložte odkaz na ten list.",
        ],
      },
      {
        id: "teams", title: "Týmy",
        items: [
          "Pokud má kvíz tabulku se živým skóre, tlačítko Načíst názvy ze živého skóre z ní přidá všechny týmy. Týmy přidejte ručně, nebo importujte CSV se sloupcem `Název týmu` (nebo `Team name`) a nepovinnými sloupci `Počet hráčů` a `Motto`. Export registrací z Google Formuláře funguje rovnou.",
          "Fotky: klikněte na čtvereček u týmu a nahrajte ji. Odkazy na fotky na Google Disku z formuláře zobrazit nejdou.",
          "Minulé výsledky: CSV s názvem týmu a jedním sloupcem za každý minulý kvíz, který má v názvu `Rank`, plus nepovinný sloupec `Accuracy`. Ukážou se na úvodní obrazovce.",
          "Názvy týmů musí odpovídat názvům v bodech, jinak v žebříčku chybí fotky a barvy.",
          "Fotky, motta a minulé výsledky jsou nepovinné. Když fotku nebo motto má méně než polovina týmů, úvodní obrazovka ukáže všechny týmy najednou jako stěnu týmů místo jedné karty po druhé; týmy bez fotky dostanou své iniciály. V nastavení show lze zvolit jedno nebo druhé natrvalo.",
        ],
      },
      {
        id: "scores", title: "Body",
        items: [
          "Možnost 1, Google Tabulka: list se sloupcem `Název týmu` (nebo `Team Name`) a jedním sloupcem pro každé kolo pojmenovaným `Kolo 1`, `Kolo 2`, … (nebo `Round 1`, …). Body jako `7,5` i `7.5` fungují. Načítá se každých 10 sekund; každý sloupec s „kolo“ nebo „round“ v názvu se počítá jako kolo, ostatní sloupce (třeba součet) se ignorují.",
          "Možnost 2, tabulka bodů: odkaz na tabulku nechte prázdný a body zapisujte do tabulky bodů (tlačítko Body v okně moderátora). Enter skočí na další tým.",
          "Žebříček vezme poslední kolo s body a nové pořadí odhaluje postupně.",
        ],
      },
      {
        id: "show", title: "Průběh show",
        items: [
          "Přetáhněte okno na televizi a stiskněte F11. Každou novou obrazovku jednou zkalibrujte přes `Shift+C`.",
          "Po úvodní obrazovce následují pravidla a ceny. Vlastní pravidla a co vyhraje které místo zadáte v nastavení kvízu (Pravidla a ceny), nebo obrazovku vypnete.",
          "Na notebooku stiskněte `Shift+P`: okno moderátora ovládá televizi a ukazuje odpovědi.",
          "Před začátkem otevřete Docházku (tlačítko v okně moderátora): zaškrtněte přítomné týmy, opravte počty hráčů a každý tým umístěte na mapu sálu (180° nebo 270° kolem vás). Při odhalování žebříčku pak okno moderátora ukáže, kam se k týmu otočit.",
          "Každé kolo má dvě tlačítka: Otázky (jen otázky s časovačem, na psaní) a Odpovědi (projde kolo znovu a odhalí odpovědi).",
          "Klávesy: `→` další krok, `←` zpět, `Z` zvětšit obrázek, `Esc` zpět na přehled, `Shift+D` přehled, `Shift+C` kalibrace, `Shift+P` moderátor.",
        ],
      },
    ],
  },

  sk: {
    title: "Hogyan készíts kvízt",
    templates: {
      title: "Sablonok",
      questions: "Kérdés sablon (CSV)",
      scores: "Pontszám sablon (CSV)",
      teams: "Csapat sablon (CSV)",
      google: "Google Táblázat sablon másolása",
      importHint: "A Google Táblázatokban: Fájl → Importálás → Feltöltés, majd válaszd ki a CSV-t.",
    },
    sections: [
      {
        id: "start", title: "Első lépések", ordered: true,
        items: [
          "Írd meg a kérdéseket a Google Táblázatokban (vagy bármilyen táblázatkezelőben) a sablon oszlopaival.",
          "A könyvtárban kattints az Új kvíz gombra, illeszd be a táblázat linkjét (vagy töltsd fel a CSV-t), és húzd be a képeket.",
          "Kapcsolj egy pontszám táblázatot (vagy a kvíz alatt írd a pontokat a pont táblázatba), aztán add hozzá a csapatokat: táblázattal a Nevek betöltése az élő pontszámokból gomb kitölti őket.",
          "Nyisd meg az Ellenőrzést: javíts ki mindent, ami piros, és nézd át a kérdések előnézetét.",
          "Indítsd el a show-t a tévén, és a laptopon nyomd meg a `Shift+P`-t a műsorvezetői ablakhoz.",
        ],
        note: "A kvízek csak ebben a böngészőben vannak elmentve. Az Exportálás (.zip) biztonsági mentésre vagy másik gépre vitelre való.",
      },
      {
        id: "questions", title: "A kérdések táblázata",
        intro: "Egy sor egy kérdés. Az első sor a fejléc, ezt kihagyja. Az oszlopokat a sorrendjük alapján olvassa, ezért ne változtass a sorrenden.",
        table: {
          head: ["Oszlop", "Mi kerül bele"],
          rows: [
            ["A `Kolo` / `Round`", "Minden kör első kérdésénél: `Round 1 - A kör címe` (vagy `Kolo 1 - …`). A többi sorban üres."],
            ["B `Q#`", "A kérdés száma, nagyban jelenik meg. Minden kérdéssornak kell szám."],
            ["C `Otázka`", "A kérdés szövege."],
            ["D `Typ Odpovědi`", "A kérdés típusa (lásd lent). Üresen Written."],
            ["E–H `A B C D`", "A négy lehetőség, csak ABCD-hez."],
            ["I `Správná odpověď`", "A felfedéskor megjelenő válasz."],
            ["J `Zdroj`", "A kép, hang vagy videó fájlneve, pl. `parizs.jpg`. Egyeznie kell egy feltöltött fájllal."],
          ],
        },
        note: "Azok a körök, amelyek címében TEST szerepel, vagy `Kolo X` a nevük, nem jelennek meg a show-ban; kipróbálásra valók.",
      },
      {
        id: "types", title: "Kérdéstípusok",
        table: {
          head: ["Típus", "A képernyőn", "Kitöltendő"],
          rows: [
            ["`Written`", "A kérdés, aztán a válasz.", "Válasz."],
            ["`Numeric`", "Mint a Written, számot kér.", "Egy szám a válasz."],
            ["`ABCD`", "Négy lehetőség; a helyes kivilágosodik.", "A–D lehetőségek, válasz `A`, `B`, `C` vagy `D`."],
            ["`Yes/No`", "Yes és No csempe; a helyes kivilágosodik.", "Válasz `Yes` vagy `No` (angolul)."],
            ["`Image`", "Kép a kérdés alatt (kattintásra vagy Z-re felnagyítható).", "Fájl a Zdroj oszlopban."],
            ["`PImage`", "Először a kép teljes képernyőn, aztán a kérdés.", "Fájl a Zdroj oszlopban."],
            ["`Audio`", "Lejátszó a kérdés alatt.", "Fájl a Zdroj oszlopban."],
            ["`Video`", "Videólejátszó a kérdés alatt.", "Fájl a Zdroj oszlopban."],
            ["`PVideo`", "Először a videó teljes képernyőn (magától indul), aztán a kérdés.", "Fájl a Zdroj oszlopban."],
            ["`Top5`", "Öt rejtett válasz, egyszerre fedi fel őket.", "Az első válasz a kérdés sorában; a többi négy a következő sorokban, csak egy számmal a Q#-ben és a válasszal."],
            ["`Sort`", "2–10 elem összekeverve, A, B, C… betűkkel; a felfedéskor sorba rendeződnek, és megjelenik a betűk sorrendje, pl. `C A D B`. Ha a Zdroj oszlopban képek vannak, az elemek képkártyák.", "Az elemek helyes sorrendben, az első a kérdés sorában, a többi a következő sorokban, csak egy számmal a Q#-ben. Minden sorban: az elem az `A` oszlopban, a betű, amely alatt megjelenik, a válasz oszlopban (A, B, C…, mindegyik egyszer, összekeverve), és ha kell, kép a Zdrojban. A válasz oszlop így fentről lefelé a választ mutatja, pl. `B D A C`, javításra készen. Betűk nélkül a show maga keveri össze az elemeket, és az Ellenőrzés kiírja, milyen betűket választott."],
          ],
        },
        examples: {
          "Written": {
            "text": "Mi Franciaország fővárosa?",
            "answer": "Párizs"
          },
          "Numeric": {
            "text": "Hány lába van a póknak?",
            "answer": "8"
          },
          "ABCD": {
            "text": "Melyik a legnagyobb bolygó?",
            "options": {
              "A": "Mars",
              "B": "Jupiter",
              "C": "Vénusz",
              "D": "Merkúr"
            },
            "answer": "B"
          },
          "Yes/No": {
            "text": "Emlős a bálna?",
            "answer": "Yes"
          },
          "Image": {
            "text": "Hol és mikor készült a fotó?",
            "answer": "Berlin, 1989"
          },
          "PImage": {
            "text": "Melyik városban történt?",
            "answer": "Párizs"
          },
          "Audio": {
            "text": "Mi ez a szignál?",
            "answer": "Quiz Night szignál"
          },
          "Video": {
            "text": "Mi látható a videón?",
            "answer": "Tesztábra"
          },
          "PVideo": {
            "text": "Hány másodperces volt a videó?",
            "answer": "5"
          },
          "Sort": {
            "text": "Rakd sorrendbe a találmányokat, a legrégebbivel kezdve",
            "items": ["Könyvnyomtatás", "Gőzgép", "Telefon", "Repülőgép", "Internet"],
            "letters": ["C", "A", "E", "B", "D"]
          },
          "Top5": {
            "text": "Sorold fel a világ 5 legnagyobb országát",
            "answer": [
              "Oroszország",
              "Kanada",
              "Kína",
              "USA",
              "Brazília"
            ]
          }
        },
        mediaStep: "Először a média", previewHint: "Minden típusnál ott vannak a sorok, amelyeket a kérdések táblázatába írsz, és hogy hogyan néznek ki a képernyőn. Kattints egy előnézetre a nagyításhoz.", sheetLabel: "A táblázatban",
      },
      {
        id: "media", title: "Képek és média",
        items: [
          "A Zdroj oszlop fájlneveinek egyezniük kell a feltöltött fájlokkal. A kis- és nagybetű és a mappák nem számítanak.",
          "Képek: jpg, png, webp, gif, avif. Hang: mp3, m4a, wav, ogg. Videó: mp4 (H.264) vagy webm; az iPhone .mov fájljai nem minden böngészőben játszhatók le.",
          "A képek legyenek nagyjából 2 MB alatt; a nagy fájlok lassítják a kvízt és az exportot.",
          "A kvíz beállításainál a média részre egész mappát is behúzhatsz. Az Ellenőrzés kilistázza a hiányzó fájlokat.",
          "A média lehet webes link is (`https://…`) egy máshol tárolt képre, de akkor a show alatt internet kell.",
        ],
      },
      {
        id: "google", title: "Google Táblázatok",
        items: [
          "Másold le a sablont (fenti gomb), vagy importáld a kérdés sablon CSV-t egy új táblázatba.",
          "Megosztás → Általános hozzáférés → Bárki, aki rendelkezik a linkkel (megtekintő). Enélkül az alkalmazás nem tudja olvasni.",
          "A linket a címsorból másold, amikor a megfelelő lap van nyitva: a `#gid=…` rész mondja meg, melyik lapot olvassa. Enélkül az első lapot olvassa. A Közzététel az interneten CSV linkje is működik.",
          "A kérdések minden show indításkor újratöltődnek, így az elírások az utolsó pillanatig javíthatók (utána töltsd újra a show-t).",
          "Ugyanaz a fájl tartalmazhat egy második lapot a pontokkal (lásd Pontok) — annak a lapnak a linkjét illeszd a pontszám mezőbe.",
        ],
      },
      {
        id: "teams", title: "Csapatok",
        items: [
          "Ha a kvíznek van élő pontszám táblázata, a Nevek betöltése az élő pontszámokból gomb az összes csapatot hozzáadja belőle. A csapatokat add hozzá kézzel, vagy importálj egy CSV-t `Název týmu` (vagy `Team name`) oszloppal és nem kötelező `Počet hráčů` / `Players` és `Motto` / `Quote` oszlopokkal. A Google Űrlap regisztrációs exportja így ahogy van, működik.",
          "Fotók: kattints a csapat melletti négyzetre a feltöltéshez. Az űrlapból származó Google Drive linkek nem jeleníthetők meg.",
          "Korábbi eredmények: CSV a csapatnévvel és minden korábbi kvízhez egy oszloppal, amelynek nevében `Rank` szerepel, valamint nem kötelező `Accuracy` oszloppal. A nyitóképernyőn jelennek meg.",
          "A csapatneveknek egyezniük kell a pontszámokban szereplő nevekkel, különben a ranglistán hiányoznak a fotók és a színek.",
          "A fotók, mottók és korábbi eredmények nem kötelezők. Ha a csapatok kevesebb mint felének van fotója vagy mottója, a nyitóképernyő egyszerre mutatja az összes csapatot csapatfalként egyenkénti kártyák helyett; a fotó nélküli csapatok a kezdőbetűiket kapják. A show beállításaiban bármelyik elrendezés kiválasztható.",
        ],
      },
      {
        id: "scores", title: "Pontok",
        items: [
          "1. lehetőség, Google Táblázat: egy lap `Team Name` (vagy `Název týmu`) oszloppal és körönként egy `Round 1`, `Round 2`, … (vagy `Kolo 1`, …) nevű oszloppal. A `7,5` és a `7.5` is működik. 10 másodpercenként frissül; minden oszlop, amelynek nevében „round” vagy „kolo” szerepel, körnek számít, a többi oszlop (például az összeg) nem számít.",
          "2. lehetőség, pont táblázat: hagyd üresen a táblázat linkjét, és írd a pontokat a pont táblázatba (Pontok gomb a műsorvezetői ablakban). Az Enter a következő csapatra ugrik.",
          "A ranglista a legutolsó pontozott kört veszi, és az új sorrendet lépésenként fedi fel.",
        ],
      },
      {
        id: "show", title: "A show menete",
        items: [
          "Húzd az ablakot a tévére, és nyomd meg az F11-et. Minden új képernyőt egyszer kalibrálj a `Shift+C`-vel.",
          "A nyitóképernyő után a szabályok és a nyeremények jönnek. A saját szabályokat és a helyezések nyereményeit a kvíz beállításainál adod meg (Szabályok és nyeremények), vagy kikapcsolod a képernyőt.",
          "A laptopon nyomd meg a `Shift+P`-t: a műsorvezetői ablak irányítja a tévét és mutatja a válaszokat.",
          "Kezdés előtt nyisd meg a Jelenlétet (gomb a műsorvezetői ablakban): jelöld be a jelenlévő csapatokat, javítsd a létszámot, és helyezd el a csapatokat a terem térképén (180° vagy 270° körülötted). A ranglista felfedésekor a műsorvezetői ablak megmutatja, merre fordulj az egyes csapatokhoz.",
          "Minden körnek két gombja van: Kérdések (csak kérdések időzítővel, az íráshoz) és Válaszok (újra végigmegy és felfedi őket).",
          "Billentyűk: `→` következő lépés, `←` vissza, `Z` kép nagyítása, `Esc` vissza az áttekintőhöz, `Shift+D` áttekintő, `Shift+C` kalibrálás, `Shift+P` műsorvezető.",
        ],
      },
    ],
  },
};
