# Jak používat Quiz Night OS

[English](USAGE.md) · [Čeština](USAGE.cs.md) · [Slovenčina](USAGE.sk.md)

Quiz Night OS běží v prohlížeči. Otevřete [aplikaci](https://jakub-kos.github.io/QuizNightWebApp/);
nic se neinstaluje a není potřeba účet. Kvízy se ukládají v prohlížeči, ve kterém jste je vytvořili,
takže si kvíz exportujte jako `.zip`, když ho chcete zálohovat nebo přenést na jiný počítač.
Používejte Chrome nebo Edge.

## 1. Vytvoření kvízu

Začněte v **knihovně kvízů**. Klikněte na **Nový kvíz**, nebo si nejdřív prohlédněte vestavěné demo.
Stránka nastavení kvízu má nahoře seznam, který ukazuje, co ještě chybí.

| Část                   | Co udělat                                                                                             |
| ---------------------- | ----------------------------------------------------------------------------------------------------- |
| **Vzhled**             | Vyberte Klasický, Halloween nebo Vánoce, importujte motiv, nebo otevřete editor motivu.               |
| **Otázky**             | Napište je v editoru otázek přímo v aplikaci, nebo připojte Google tabulku (viz níže).                |
| **Obrázky a média**    | Přetáhněte obrázky, zvuky a videa, na která se otázky odkazují. Jde přetáhnout i celá složka.         |
| **Týmy**               | Zapište je, importujte CSV, nebo načtěte názvy z tabulky s body. Fotka a motto jsou nepovinné.        |
| **Živé skóre**         | Připojte tabulku s body, nebo body zapisujte do aplikace během večera.                                |
| **Pravidla a ceny**    | Upravte obrazovky s pravidly a cenami, nebo kteroukoli z nich vypněte.                                |

### Otázky z Google tabulky

1. Zkopírujte si šablonu tabulky (Nápověda → Šablony) a vyplňte ji.
2. Nasdílejte tabulku jako _každý, kdo má odkaz, může zobrazit_.
3. Vložte odkaz do části Otázky.

Nápověda → Typy otázek ukazuje přesné řádky pro každý typ a jak vypadá na televizi. Typy jsou:
výběr z možností, ano nebo ne, číslo, psaná odpověď, žebříček, seřazení, tištěný list, s obrázkem,
nejdřív obrázek, se zvukem, s videem a nejdřív video.

### Kontrola před večerem

- **Kontrola** ukáže každou otázku tak, jak bude na televizi, a vypíše, co chybí nebo je špatně.
- **Mapa** ukáže všechny obrazovky večera popořadě a co vás posune na další.

## 2. Průběh večera

1. Připojte notebook k televizi. Otevřete kvíz a klikněte na **Spustit show**.
2. Přetáhněte okno se show na televizi a stiskněte `F11` pro celou obrazovku.
3. Stiskněte `Shift + P`. Otevře se okno moderátora; nechte ho na notebooku.
4. Když televize ořezává okraje, stiskněte v okně na televizi `Shift + C` a obraz zmenšete.

Televize následuje každé kliknutí v okně moderátora. Obě okna musí být ve stejném prohlížeči na
stejném počítači.

### Pořadí večera

1. **Úvod**: každý tým dostane představení.
2. **Pravidla**, potom **Ceny** (pokud jsou zapnuté).
3. **Přehled kol**: střed celého večera. Každé kolo, žebříček i přestávka tady začínají a sem se vracejí.
4. **Kolo**: hrajte ho s odpověďmi, nebo jen otázky a odpovědi odhalte později.
5. **Žebříček**: body za kolo se odhalují po skupinách, od nejnižšího počtu nahoru.
6. **Přestávka**: odpočet s týmy a aktuálním pořadím.

### Klávesy

| Klávesa     | Co dělá                                          |
| ----------- | ------------------------------------------------ |
| `→` / `←`   | Další / předchozí krok                           |
| `Shift + P` | Otevře okno moderátora                           |
| `Shift + D` | Skočí zpět na přehled kol                        |
| `Shift + C` | Přizpůsobí velikost obrazu televizi              |
| `Z`         | Zvětší obrázek u otázky                          |
| `Esc`       | Zavře obrázek, nebo opustí aktuální obrazovku    |

### Docházka a body

- **Docházka**: před začátkem zaškrtněte týmy, které dorazily, opravte počet hráčů a každý tým
  umístěte na mapu sálu. Při odhalování žebříčku okno moderátora ukáže, kam se otočit.
- **Body**: zapište body každého týmu za každé kolo. Hned se uloží a žebříček v běžící show se sám
  aktualizuje. S připojenou tabulkou s body čte show body z ní.

## 3. Motivy

Každý kvíz si vzhled vybírá na stránce nastavení. V **editoru motivu** se mění barvy, písma, obrázek
na pozadí, záře, kulisy a ozdoby, s živým náhledem každé obrazovky a kontrolou čitelnosti. Motiv jde
exportovat, importovat a uložit do **Mých motivů** pro další kvízy.

**Dílna motivů** (odkaz je v knihovně) ukazuje motivy, které sdíleli jiní moderátoři. Vlastní motiv
odešlete v editoru motivu tlačítkem **Sdílet do dílny**; objeví se po kontrole.

## 4. Nastavení a jazyk

Ikona ozubeného kola v show otevře nastavení: jazyk, zvuky a hlasitost, rychlost animací a čas
začátku. Aplikace je v angličtině, češtině a slovenštině.

## Dobré vědět

- Kvízy žijí v jednom prohlížeči na jednom počítači. Smazáním dat webu v prohlížeči o ně přijdete. Exportujte si `.zip`.
- Připojená Google tabulka potřebuje internet. Otázky napsané v aplikaci a přetažená média ne.
- Připojená Google tabulka se upravuje v Google Tabulkách; aplikace ji jen čte.
