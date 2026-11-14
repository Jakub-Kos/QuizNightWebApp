# A Quiz Night OS használata

[English](USAGE.md) · [Čeština](USAGE.cs.md) · [Slovenčina](USAGE.sk.md)

A Quiz Night OS a böngészőben fut. Nyisd meg az [alkalmazást](https://jakub-kos.github.io/QuizNightWebApp/);
nem kell semmit telepíteni, és fiók sem kell. A kvízeid abban a böngészőben tárolódnak, amelyikben
készítetted őket, ezért exportáld a kvízt `.zip` fájlba, ha biztonsági mentést szeretnél, vagy másik
gépre vinnéd. Chrome vagy Edge böngészőt használj.

## 1. Kvíz készítése

A **kvízkönyvtárban** kezdj. Kattints az **Új kvíz** gombra, vagy előbb nézz körül a beépített demóban.
A kvíz beállítási oldalának tetején egy ellenőrzőlista mutatja, mi hiányzik még.

| Rész                   | Teendő                                                                                                        |
| ---------------------- | ------------------------------------------------------------------------------------------------------------- |
| **Megjelenés**         | Válaszd a Klasszikus, Halloween vagy Karácsony témát, importálj egyet, vagy nyisd meg a témaszerkesztőt.      |
| **Kérdések**           | Írd meg őket az alkalmazás kérdésszerkesztőjében, vagy kapcsolj hozzá egy Google-táblázatot (lásd lent).      |
| **Képek és média**     | Húzd be a kérdésekben megnevezett képeket, hangokat és videókat. Egész mappa is behúzható.                    |
| **Csapatok**           | Írd be őket, importálj CSV-t, vagy töltsd be a neveket a pontszámtáblázatból. A fotó és a mottó nem kötelező. |
| **Pontszámok**         | Kapcsolj hozzá egy pontszámtáblázatot, vagy írd be a pontokat az alkalmazásba az est közben.                  |
| **Szabályok és díjak** | Szerkeszd a szabályok és a díjak képernyőjét, vagy kapcsold ki bármelyiket.                                   |

### Kérdések Google-táblázatból

1. Másold le a táblázatsablont (Súgó → Sablonok), és töltsd ki.
2. Oszd meg a táblázatot úgy, hogy _a link birtokában bárki megtekintheti_.
3. Illeszd be a linket a Kérdések részbe.

A Súgó → Kérdéstípusok megmutatja az egyes típusok pontos sorait, és azt is, hogyan néznek ki a tévén.
A típusok: feleletválasztós, igen vagy nem, szám, írásos válasz, toplista, sorba rendezés, nyomtatott
feladatlap, képpel, először a kép, hanggal, videóval, először a videó.

### Ellenőrzés az est előtt

- Az **Ellenőrzés** minden kérdést úgy mutat, ahogy a tévén megjelenik, és felsorolja, mi hiányzik vagy hibás.
- A **Térkép** az est összes képernyőjét sorrendben mutatja, és azt is, mi visz tovább a következőre.

## 2. Az est levezetése

1. Csatlakoztasd a laptopot a tévéhez. Nyisd meg a kvízt, és kattints a **Show indítása** gombra.
2. Húzd a show ablakát a tévére, és nyomd meg az `F11` billentyűt a teljes képernyőhöz.
3. Nyomd meg a `Shift + P` billentyűket. Megnyílik a műsorvezetői ablak; ezt hagyd a laptopon.
4. Ha a tévé levágja a kép szélét, nyomd meg a tévé ablakában a `Shift + C` billentyűket, és kicsinyítsd a képet.

A tévé a műsorvezetői ablak minden kattintását követi. A két ablaknak ugyanabban a böngészőben,
ugyanazon a gépen kell lennie.

### Az est menete

1. **Üdvözlés**: minden csapat bemutatkozik.
2. **Szabályok**, majd **Díjak** (ha be vannak kapcsolva).
3. **Körök áttekintése**: ez az est közepe. Minden kör, a ranglista és a szünet innen indul, és ide tér vissza.
4. **Egy kör**: játszható válaszokkal együtt, vagy csak kérdésekkel, a válaszokat később felfedve.
5. **Ranglista**: a kör pontjait csoportonként fedi fel, a legalacsonyabb pontszámtól felfelé.
6. **Szünet**: visszaszámlálás a csapatokkal és az élő állással.

### Billentyűk

| Billentyű   | Mit csinál                                          |
| ----------- | --------------------------------------------------- |
| `→` / `←`   | Következő / előző lépés                             |
| `Shift + P` | Megnyitja a műsorvezetői ablakot                    |
| `Shift + D` | Visszaugrik a körök áttekintésére                   |
| `Shift + C` | A kép méretének igazítása a tévéhez                 |
| `Z`         | Kinagyítja a kérdés képét                           |
| `Esc`       | Bezárja a képet, vagy kilép az aktuális képernyőről |

### Jelenlét és pontszámok

- **Jelenlét**: kezdés előtt jelöld be a megérkezett csapatokat, javítsd a játékosok számát, és helyezd
  el minden csapatot a terem térképén. A ranglista felfedésekor a műsorvezetői ablak megmutatja, merre nézz.
- **Pontszámok**: írd be minden csapat pontjait körönként. Azonnal mentődnek, és a futó show ranglistája
  magától frissül. Ha pontszámtáblázat van hozzákapcsolva, a show abból olvas.

## 3. Témák

Minden kvíz a beállítási oldalon választ megjelenést. A **témaszerkesztőben** állíthatók a színek, a
betűtípusok, a háttérkép, a fények, a díszlet és a díszítések; minden képernyőről élő előnézet látszik,
és van olvashatósági ellenőrzés is. A téma exportálható, importálható, és elmenthető a **Saját témák**
közé más kvízekhez.

A **témaműhely** (a könyvtárból érhető el) más műsorvezetők megosztott témáit mutatja. A sajátodat a
témaszerkesztő **Megosztás a műhelyben** gombjával küldheted be; ellenőrzés után jelenik meg.

## 4. Beállítások és nyelv

A show-ban a fogaskerék ikon nyitja meg a beállításokat: nyelv, hangok és hangerő, animáció sebessége
és a kezdési időpont. Az alkalmazás angolul, csehül és szlovákul érhető el.

## Jó tudni

- A kvízek egyetlen gép egyetlen böngészőjében élnek. A böngésző webhelyadatainak törlése őket is törli. Exportálj `.zip` fájlt.
- A hozzákapcsolt Google-táblázathoz internet kell. Az alkalmazásban írt kérdésekhez és a behúzott médiához nem.
- A hozzákapcsolt Google-táblázatot a Google Táblázatokban kell szerkeszteni; az alkalmazás csak olvassa.
