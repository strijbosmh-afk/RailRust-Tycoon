"""Build the offline player manual using only Python's standard library.

The source deliberately uses a small Markdown subset: headings, paragraphs,
flat lists, tables, fenced code, emphasis and links. No raw HTML is accepted.
Run with --check in CI to reject stale output or broken local anchors.
"""
from __future__ import annotations

import argparse
import html
import re
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "docs" / "PLAYER_MANUAL.md"
OUTPUT = SOURCE.with_suffix(".html")
LEGACY_OUTPUT = ROOT / "docs" / "HANDLEIDING_NL.html"


def slug(text: str) -> str:
    text = unicodedata.normalize("NFKD", text).encode("ascii", "ignore").decode().lower()
    return re.sub(r"[^a-z0-9]+", "-", text.replace("'", "").replace("’", "")).strip("-")


def inline(text: str) -> str:
    # First escape all author input. Only the constructs below create HTML.
    text = html.escape(text, quote=True)
    code: list[str] = []

    def keep_code(match: re.Match[str]) -> str:
        code.append(f"<code>{match[1]}</code>")
        return f"\x00{len(code) - 1}\x00"

    text = re.sub(r"`([^`]+)`", keep_code, text)
    text = re.sub(r"\[([^\]]+)\]\((#[a-z0-9-]+)\)", r'<a href="\2">\1</a>', text)
    text = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", text)
    return re.sub(r"\x00(\d+)\x00", lambda m: code[int(m[1])], text)


def render(source: str) -> tuple[str, list[tuple[int, str, str]]]:
    lines = source.splitlines()
    out: list[str] = []
    headings: list[tuple[int, str, str]] = []
    section_open = False
    i = 0
    while i < len(lines):
        line = lines[i]
        if not line.strip():
            i += 1
            continue
        if line.startswith("# "):
            out.append(f"<header class=hero><p class=eyebrow>PLAY · BUILD · MANAGE</p><h1>{inline(line[2:])}</h1></header>")
            i += 1
        elif line.startswith("## ") or line.startswith("### "):
            level = 3 if line.startswith("### ") else 2
            title = line[level + 1:]
            anchor = slug(title)
            if any(h[1] == anchor for h in headings):
                raise ValueError(f"Duplicate heading: {anchor}")
            headings.append((level, anchor, title))
            if level == 2:
                if section_open:
                    out.append('<a class="back" href="#top">↑ Contents and top</a></section>')
                out.append(f'<section aria-labelledby="{anchor}">')
                section_open = True
            out.append(f'<h{level} id="{anchor}" tabindex="-1">{inline(title)}</h{level}>')
            i += 1
        elif line.startswith("```"):
            i += 1
            code = []
            while i < len(lines) and not lines[i].startswith("```"):
                code.append(lines[i])
                i += 1
            if i == len(lines):
                raise ValueError("Unclosed code block")
            out.append("<pre><code>" + html.escape("\n".join(code)) + "</code></pre>")
            i += 1
        elif line.startswith("| "):
            rows = []
            while i < len(lines) and lines[i].startswith("| "):
                rows.append([cell.strip() for cell in lines[i].strip("|").split("|")])
                i += 1
            if len(rows) < 2 or not all(re.fullmatch(r":?-+:?", c) for c in rows[1]):
                raise ValueError("Table needs a header separator")
            if any(len(row) != len(rows[0]) for row in rows):
                raise ValueError("Inconsistent table columns")
            out.append('<div class="table-scroll" role="region" aria-label="Table" tabindex="0"><table><thead><tr>')
            out.extend(f'<th scope="col">{inline(c)}</th>' for c in rows[0])
            out.append("</tr></thead><tbody>")
            for row in rows[2:]:
                out.append("<tr>" + "".join(f"<td>{inline(c)}</td>" for c in row) + "</tr>")
            out.append("</tbody></table></div>")
        elif re.match(r"^(\d+\. |[-*] )", line):
            ordered = bool(re.match(r"^\d+\. ", line))
            tag = "ol" if ordered else "ul"
            pattern = r"^\d+\. " if ordered else r"^[-*] "
            out.append(f"<{tag}>")
            while i < len(lines) and re.match(pattern, lines[i]):
                out.append("<li>" + inline(re.sub(pattern, "", lines[i])) + "</li>")
                i += 1
            out.append(f"</{tag}>")
        else:
            paragraph = [line]
            i += 1
            while i < len(lines) and lines[i].strip() and not re.match(r"^(#|```|\| |\d+\. |[-*] )", lines[i]):
                paragraph.append(lines[i])
                i += 1
            out.append("<p>" + inline(" ".join(paragraph)) + "</p>")
    if section_open:
        out.append('<a class="back" href="#top">↑ Contents and top</a></section>')
    anchors = {anchor for _, anchor, _ in headings} | {"top"}
    for anchor in re.findall(r'href="#([a-z0-9-]+)"', "".join(out)):
        if anchor not in anchors:
            raise ValueError(f"Broken manual link: #{anchor}")
    if len([h for h in headings if h[0] == 2]) != 25:
        raise ValueError("Expected all 25 manual chapters")
    return "\n".join(out), headings


STYLE = """
:root{color-scheme:light;--ink:#163b31;--green:#123c31;--gold:#f5ce68;--paper:#fffdf4;--line:#d9e2d5}
*{box-sizing:border-box}html{scroll-behavior:smooth;scroll-padding-top:1.5rem}
body{margin:0;background:#eaf0e7;color:var(--ink);font:17px/1.7 system-ui,Segoe UI,sans-serif}
a{color:#116a50;text-underline-offset:.2em}a:hover{color:#854d05}a:focus-visible,button:focus-visible,input:focus-visible,summary:focus-visible,[tabindex]:focus-visible{outline:3px solid #cd8c09;outline-offset:4px}
.skip{position:fixed;top:-100px;background:var(--gold);z-index:10;padding:10px}.skip:focus{top:0}
.layout{max-width:1580px;margin:auto;display:grid;grid-template-columns:310px minmax(0,1fr);gap:28px;padding:28px}
aside{position:sticky;top:20px;height:calc(100vh - 40px);overflow:auto;padding:22px;background:var(--green);color:#fff9df;border-radius:18px;scrollbar-color:#9cb7a8 var(--green)}
summary{font-size:1.3rem;font-weight:750;cursor:pointer}.search{margin:18px 0}label{display:block;font-weight:650}input{display:block;width:100%;font:inherit;padding:10px;border:2px solid #e3c572;border-radius:8px;background:#fffdf5;color:#163b31;margin:6px 0}
button{font:inherit;cursor:pointer;padding:6px 12px;border:1px solid #e5cc8b;border-radius:7px;background:#f5ce68;color:#203b2e}
.controls{display:flex;flex-wrap:wrap;gap:8px}.status{font-size:.83rem;color:#e1e8d9;margin:8px 0;min-height:1.6em}
nav ul{list-style:none;padding:0;margin:12px 0}nav li{margin:3px 0}nav a{display:block;color:#fff7d4;text-decoration:none;font-size:.88rem;line-height:1.4;padding:8px;border-radius:5px;border-left:3px solid transparent}
nav .sub a{font-size:.79rem;color:#c4dbcc;padding:4px 8px 4px 21px}nav a:hover,nav a[aria-current]{background:#285344;border-color:var(--gold);color:#fff}
main{min-width:0;max-width:1050px}.hero{background:var(--green);color:#fff8dd;border:1px solid #426a55;padding:34px;border-radius:18px}.eyebrow{font-size:.8rem;letter-spacing:.16em;color:var(--gold);font-weight:700}h1{font-size:clamp(1.8rem,3vw,2.8rem);line-height:1.2;margin:10px 0}
section{background:var(--paper);border:1px solid var(--line);border-radius:14px;padding:28px 34px;margin:24px 0;box-shadow:0 3px 12px #183e3006}
h2{font-size:1.6rem;line-height:1.3;margin:0 0 22px;border-bottom:3px solid #e4b747;padding-bottom:16px}h3{font-size:1.16rem;margin:32px 0 12px;color:#17674d}p{margin:15px 0}li{padding:4px 0 4px 6px}ol li::marker{font-weight:800;color:#126a4d}strong{color:#174e39}code{background:#edf1e7;border-radius:4px;padding:.1em .25em;font-size:.86em;overflow-wrap:anywhere}
pre{overflow:auto;background:#e9f0e6;padding:18px;border-radius:10px;line-height:1.6}pre code{background:none;white-space:pre;overflow-wrap:normal;padding:0}.table-scroll{overflow-x:auto;margin:18px 0}table{border-collapse:collapse;width:100%;font-size:.9rem}th{text-align:left;background:#dae8d6;color:#153e2c}th,td{padding:12px;border:1px solid #d8e0d2;vertical-align:top}tbody tr:nth-child(even){background:#f1f4e9}
.back{display:inline-block;margin-top:18px;font-size:.85rem}mark{background:#ffe28b;color:#173d30;border-radius:2px}.no-results{background:#fff5d5;border:1px solid #cea336;border-radius:10px;padding:24px;margin-top:24px}[hidden]{display:none!important}footer{padding:24px;font-size:.85rem}#position{position:fixed;top:0;left:0;height:4px;background:#c48b12;width:0;z-index:9}
@media(max-width:950px){.layout{display:block;padding:12px}aside{position:static;height:auto;margin-bottom:18px;padding:18px}nav{max-height:38vh;overflow:auto}.hero{padding:24px}section{padding:24px 20px}body{font-size:16px}}
@media(prefers-reduced-motion:reduce){html{scroll-behavior:auto}}
@media print{body{background:white;color:black;font:11pt/1.5 sans-serif}.layout{display:block;padding:0}aside,.back,.no-results,#position,.skip{display:none!important}main{max-width:none}.hero{background:white;color:black;border:0;padding:0}.eyebrow{color:black}section{display:block!important;border:0;box-shadow:none;padding:0;margin:20pt 0;break-before:page}table{font-size:9pt}tr{break-inside:avoid}h2,h3{break-after:avoid}a{color:inherit;text-decoration:none}mark{background:none}}
"""

SCRIPT = """
'use strict';
const search=document.querySelector('#search'),status=document.querySelector('#status');
const sections=[...document.querySelectorAll('main section')],links=[...document.querySelectorAll('nav a')];
const normalize=s=>s.normalize('NFD').replace(/[\\u0300-\\u036f]/g,'').toLocaleLowerCase('en');
const index=sections.map(el=>normalize(el.textContent));
function unmark(){document.querySelectorAll('mark').forEach(m=>m.replaceWith(document.createTextNode(m.textContent)));sections.forEach(s=>s.normalize());}
function highlight(section,query){
  if(!query)return;const walk=document.createTreeWalker(section,NodeFilter.SHOW_TEXT);const nodes=[];
  while(walk.nextNode())nodes.push(walk.currentNode);
  for(const node of nodes){const text=node.textContent,lower=normalize(text),at=lower.indexOf(query);if(at<0)continue;
    // Highlight only equal-length normalized text (diacritics); never inject a search string as HTML.
    if(lower.length!==text.length)continue;
    const fragment=document.createDocumentFragment();let start=0,pos=at;
    while(pos>=0){fragment.append(document.createTextNode(text.slice(start,pos)));const mark=document.createElement('mark');mark.textContent=text.slice(pos,pos+query.length);fragment.append(mark);start=pos+query.length;pos=lower.indexOf(query,start);}
    fragment.append(document.createTextNode(text.slice(start)));node.replaceWith(fragment);
  }
}
function filter(){unmark();const query=normalize(search.value.trim()).slice(0,100),terms=query.split(/\\s+/).filter(Boolean);let count=0;
  sections.forEach((s,i)=>{s.hidden=!terms.every(t=>index[i].includes(t));if(!s.hidden){count++;terms.forEach(t=>highlight(s,t));}});
  links.forEach(a=>{const target=document.getElementById(a.hash.slice(1));a.closest('li').hidden=target.closest('section').hidden;});
  status.textContent=query?`${count} of ${sections.length} chapters found`:`${sections.length} chapters · offline`;
  document.querySelector('#no-results').hidden=count!==0;
}
search.addEventListener('input',filter);document.querySelector('#clear').addEventListener('click',()=>{search.value='';filter();search.focus();});
document.querySelector('#print').addEventListener('click',()=>{search.value='';filter();window.print();});
document.addEventListener('keydown',e=>{if(e.key==='/'&&!['INPUT','TEXTAREA'].includes(document.activeElement.tagName)){e.preventDefault();search.focus();}if(e.key==='Escape'&&document.activeElement===search){search.value='';filter();}});
document.querySelectorAll('a[href^="#"]').forEach(a=>a.addEventListener('click',()=>{if(search.value){search.value='';filter();}}));
window.addEventListener('hashchange',()=>{const target=document.getElementById(location.hash.slice(1));if(target){if(search.value){search.value='';filter();}target.scrollIntoView();}});
if('IntersectionObserver' in window){const observer=new IntersectionObserver(entries=>{for(const entry of entries){if(entry.isIntersecting){links.forEach(a=>a.removeAttribute('aria-current'));const a=links.find(a=>a.hash==='#'+entry.target.id);if(a)a.setAttribute('aria-current','location');}}},{rootMargin:'0px 0px -65% 0px'});document.querySelectorAll('h2,h3').forEach(h=>observer.observe(h));}
window.addEventListener('scroll',()=>{const total=document.documentElement.scrollHeight-innerHeight;document.querySelector('#position').style.width=(total>0?scrollY/total*100:0)+'%';},{passive:true});
filter();
"""


def build() -> str:
    body, headings = render(SOURCE.read_text(encoding="utf-8"))
    toc = "\n".join(
        f'<li class="{"sub" if level == 3 else "chapter"}"><a href="#{anchor}">{html.escape(title)}</a></li>'
        for level, anchor, title in headings
    )
    return f'''<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; base-uri 'none'; form-action 'none'">
<meta name="description" content="Parkbound: complete offline player manual with step-by-step building, management, campaigns and troubleshooting.">
<title>Parkbound — player manual 0.99.3</title><style>{STYLE}</style></head>
<body id="top"><a class="skip" href="#content">Skip to the manual</a><div id="position" aria-hidden="true"></div><div class="layout">
<aside><details open><summary>Contents &amp; search</summary><div class="search"><label for="search">Find a topic</label><input id="search" type="search" maxlength="100" placeholder="Try queue or budget" autocomplete="off" aria-controls="content"><div class="controls"><button id="clear" type="button">Clear search</button><button id="print" type="button">Print / PDF</button></div><p class="status" id="status" role="status" aria-live="polite">25 chapters · offline</p></div><nav aria-label="Chapters"><ul>{toc}</ul></nav></details></aside>
<main id="content"><noscript><p>Search requires JavaScript. All chapters, links and browser printing remain available without it.</p></noscript>{body}<div class="no-results" id="no-results" hidden>No chapters found. Try a shorter term, such as path, price or stock.</div><footer>Parkbound 0.99.3 · Generated from PLAYER_MANUAL.md · No internet or account required.</footer></main></div><script>{SCRIPT}</script></body></html>
'''


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Check source links and generated HTML freshness")
    args = parser.parse_args()
    expected = build()
    if args.check:
        if any(not path.is_file() or path.read_text(encoding="utf-8") != expected
               for path in (OUTPUT, LEGACY_OUTPUT)):
            raise SystemExit("Player manual is stale. Run python scripts/build-player-manual.py")
        print("Player manual: 25 chapters, anchors valid, offline HTML up to date.")
    else:
        for path in (OUTPUT, LEGACY_OUTPUT):
            path.write_text(expected, encoding="utf-8", newline="\n")
        print(f"Built {OUTPUT}")


if __name__ == "__main__":
    main()
