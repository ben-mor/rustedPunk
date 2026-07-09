# Wiki-Update-Entwürfe

> **Status: ANGEWENDET am 2026-07-09** (Bot-Account „Morast“ via
> `tools/fandom.py`). Punkte 1–5 wie unten; Punkt 6 bewusst konservativer:
> nur der Schadens-Bullet wurde geändert (5D6 + Czar-Hinweis), Reichweite
> 300 m und 1/15 Schuss blieben unangetastet, weil sie von Ben stammen
> könnten. Die Snapshots in diesem Ordner sind auf dem neuen Stand.
> Datei bleibt als Vorlage für künftige Regel-Synchronisierungen.

Fertige Wikitext-Schnipsel für desaster.fandom.com, damit das Wiki wieder mit
den entschiedenen Regeln (PROJECT-STRUCTURE.md §3, Stand 2026-07-09)
übereinstimmt. Pro Abschnitt: Seite, was ersetzt wird, neuer Text.

---

## 1. Seite `Hausregeln`, Abschnitt `== Würfel ==`

**Ersetzt:** den Satz „Ein Wurf von 10 ist eine besondere Leistung in dieser
Fertigkeit. Ein CP für diese Fertigkeit und nochmal würfeln und dazu zählen
(kaskadiert)." — die CP-Regel ist entfallen (wird am Tisch nicht gespielt).

```
Ein Wurf von 10 ist eine besondere Leistung: nochmal würfeln und dazu zählen (kaskadiert).
```

**Neu anfügen (Glück, drei Ebenen + Regeneration):**

```
=== Glück ===
Glückspunkte werden VOR dem Wurf eingesetzt und verändern den Würfel direkt: eine gewürfelte 9 mit einem Glückspunkt zählt als natürliche 10 (und kaskadiert), eine gewürfelte 1 mit einem Glückspunkt zählt als 2 und ist kein Patzer.

Glück hat drei Ebenen:
# '''Start-Basis''': der Wert aus der Charaktererschaffung, ändert sich nie.
# '''Aktuelle Basis''': kann für extreme „Die Welt dreht sich jetzt zu deinen Gunsten"-Ereignisse dauerhaft gesenkt werden. Regeneration und Obergrenze richten sich nach diesem Wert.
# '''Aktueller Pool''': schwankt mit jedem Einsatz und bleibt über Sitzungen erhalten.

Zu Beginn jeder Sitzung regeneriert der Pool die Hälfte der aktuellen Basis (aufgerundet), maximal bis zur aktuellen Basis. Beispiel: Basis 9, davon 8 ausgegeben (1 übrig) → nächste Sitzung +5 → Start mit 6.
```

---

## 2. Seite `Hausregeln`, Abschnitt `== Schaden ==`

**Ergänzen beim Prellschaden-Absatz (Klarstellung harte Rüstung):**

```
Prellschaden entsteht nur durch Treffer, die von WEICHER Rüstung gefangen werden (und durch anstrengende Arbeiten etc.). Von harter Rüstung absorbierter Schaden erzeugt keinen Prellschaden.

Der Body Type Modifier (BTM) wird vom ankommenden Schaden abgezogen und in Prellschaden umgewandelt (Minimum 1 echter Schaden bleibt). Nahkampfwaffen und Kampfkunst benutzen beim Austeilen den davon verschiedenen Schadensmodifikator „DAM".
```

---

## 3. Seite `Hausregeln`, neuer Abschnitt `== Traglast ==`

**Neu (Tabelle war bisher nur im Code, Entscheidung Q4: Code ist richtig):**

```
== Traglast ==
Tragekapazität = BODY × 10 kg. Deadlift = das Vierfache davon.

Abzug auf Reflexe und Move nach dem Verhältnis Inventargewicht / Kapazität:
{| class="wikitable"
! Verhältnis !! Abzug
|-
| unter 0,5 || 0
|-
| 0,5 bis unter 0,7 || −1
|-
| 0,7 bis unter 1,0 || −2
|-
| 1,0 bis unter 1,3 || −4
|-
| 1,3 bis unter 1,6 || −6
|-
| ab 1,6 || −8
|}
Reflexe werden zusätzlich um die Behinderung der getragenen Rüstung reduziert, Move nur um die Inventar-Behinderung.
```

---

## 4. Seite `Regeln`, Abschnitt `==== Mehr als 8 Schaden ====`

**Ersetzt:** den BODY-Wurf-Text (obsolet, Entscheidung Q6: der Code ist richtig).

```
==== Mehr als 8 Schaden ====
Kommen in einem Treffer mehr als 8 Punkte Schaden (nach BTM) in einem Körperteil an, ist das Körperteil zerstört. Kopf-, Brust- und Vitals-Treffer sind dann sofort tödlich; bei anderen Zonen ist man ab sofort mindestens Mortal 0 (13 Schaden) und dabei zu sterben.
```

---

## 5. Seite `Regeln`, Abschnitt `=== Glück ===`

**Ersetzt:** „Man darf pro Spielabend nicht mehr Glückspunkte ausgeben als man
LUCK hat. …" durch den Drei-Ebenen-Text aus Punkt 1 (gleicher Wortlaut).

---

## 6. Seite `Technlogien`, Abschnitt `=== AK-47 ===`

**Korrektur:** 6d6+2 gehört zur Czar AK-47 / Kalashnikov A-80; die klassische
AK-47 hat laut CP2020 Reference Book 5 (und am Tisch gespielt) 5d6.

```
* Gewehr
* 5D6 Schaden (7,62 × 39 mm). Mit panzerbrechender Munition: Schutzwert wird halbiert, Schaden gegen weiche Ziele auch.
* Reichweite 400 Meter
* Magazin 30 Schuss, bis zu 20 Schuss pro Runde (Autofeuer)
* Sehr zuverlässig
* Nicht versteckbar
```

(Hinweis: „ab 8 Schadenspunkten wird von einem Durchschlag ausgegangen" im
alten Text vermischt die Durchschuss-Regel — die steht allgemein unter
[[Regeln]], Durchschlag bei Schusswaffen, und gehört nicht zur Waffe.)

(Tippfehler nebenbei: Seitenname „Technlogien" → „Technologien",
„hypotetisch" → „hypothetisch".)
