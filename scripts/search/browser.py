#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = ["playwright==1.60.0"]
# ///
"""Functional search proof against an actual generated static publication."""
import argparse
import functools
import http.server
import json
from pathlib import Path
import subprocess
import tempfile
import threading
from playwright.sync_api import sync_playwright, expect


HIGHLIGHT = 'span.ui__list-item-highlighted-span'


def complete(page, action):
    """Run `action` and wait until the dialog finishes drawing its result."""
    before = page.evaluate('Number(document.querySelector("#lsg-search-results")?.dataset.lsgSearchComplete || 0)')
    action()
    page.wait_for_function('(before) => Number(document.querySelector("#lsg-search-results").dataset.lsgSearchComplete) > before', arg=before)


def group_names(page):
    return page.locator('#lsg-search-results .lsg-search-group-name').all_inner_texts()


def grouped_dialog(browser, base):
    """Logseq's groups, rows, filters, commands, keyboard and look."""
    context = browser.new_context(viewport={'width': 1280, 'height': 800}, color_scheme='light')
    page = context.new_page()
    errors = []
    page.on('pageerror', lambda error: errors.append(str(error)))
    requests = []
    page.on('request', lambda request: requests.append(request.url))
    page.goto(base + '/index.html')
    trigger = page.locator('[data-lsg-search]')
    dialog = page.locator('#lsg-search-dialog')
    field = page.locator('#lsg-search-input')
    options = page.locator('#lsg-search-results [role=option]')
    # Pointing at the button loads the dialog module, manifest and title index, nothing more.
    expect(trigger).to_be_visible()
    assert not any('lsg-search-ui-' in url for url in requests)
    trigger.hover()
    page.wait_for_function('() => performance.getEntriesByType("resource").some(entry => entry.name.includes("lsg-search-titles-"))')
    assert page.locator('#lsg-search-dialog').count() == 0
    assert not any('lsg-search-shard-' in url or 'lsg-search-chunk-' in url for url in requests)
    # The empty input shows Commands and no other group.
    trigger.click()
    expect(dialog).to_have_attribute('data-lsg-search-state', 'ready')
    assert group_names(page) == ['Commands'], group_names(page)
    labels = options.all_inner_texts()
    assert [label.split('\n')[0] for label in labels] == ['Toggle wide mode', 'Toggle between dark/light theme', 'Toggle left sidebar', 'Go to home', 'Go to all pages'], labels
    # Up from the first row wraps to the last; Down and Ctrl+N wrap back; Ctrl+P goes up.
    expect(options.nth(0)).to_have_attribute('aria-selected', 'true')
    page.keyboard.press('ArrowUp')
    expect(options.nth(4)).to_have_attribute('aria-selected', 'true')
    page.keyboard.press('ArrowDown')
    expect(options.nth(0)).to_have_attribute('aria-selected', 'true')
    page.keyboard.press('Control+p')
    expect(options.nth(4)).to_have_attribute('aria-selected', 'true')
    page.keyboard.press('Control+n')
    expect(options.nth(0)).to_have_attribute('aria-selected', 'true')
    expect(page.locator('.lsg-search-hint')).to_contain_text('Trigger')
    # Enter on a command closes search and runs it.
    theme = page.evaluate('window.logseqGardenerTheme.appearance()')
    complete(page, lambda: field.fill('theme'))
    assert group_names(page) == ['Commands']
    page.keyboard.press('Enter')
    expect(dialog).not_to_be_visible()
    assert page.evaluate('window.logseqGardenerTheme.appearance()') != theme
    page.evaluate("window.logseqGardenerTheme.set('light')")
    expect(trigger).to_be_focused()
    # tm then tmux fetch the tm shard once, no other shard, and only shown rows' chunks.
    before = len(requests)
    trigger.click()
    complete(page, lambda: page.keyboard.type('t'))
    assert not any('lsg-search-shard-' in url for url in requests[before:])
    complete(page, lambda: page.keyboard.type('m'))
    complete(page, lambda: page.keyboard.type('u'))
    complete(page, lambda: page.keyboard.type('x'))
    shards = [url for url in requests[before:] if 'lsg-search-shard-' in url]
    assert len(shards) == 1, shards
    chunks = [url for url in requests[before:] if 'lsg-search-chunk-' in url]
    shown = page.locator('#lsg-search-results [data-lsg-search-group=blocks] [role=option]').count()
    assert 1 <= len(chunks) <= shown, chunks
    # Pages come before Blocks, both 99+, five rows each.
    assert group_names(page) == ['Pages', 'Blocks'], group_names(page)
    expect(page.get_by_role('button', name='Retry search')).to_be_hidden()
    assert page.locator('.lsg-search-count').all_inner_texts() == ['99+', '99+']
    pages = page.locator('#lsg-search-results [data-lsg-search-group=pages] [role=option]')
    expect(pages).to_have_count(5)
    assert pages.first.inner_text() == 'tmux'
    more = page.locator('[data-lsg-search-group=pages] .lsg-search-more')
    expect(more).to_be_visible()
    assert more.inner_text().replace('\n', ' ').split()[:2] == ['Show', 'more']
    expect(page.locator('[data-lsg-search-group=blocks] .lsg-search-more')).to_be_hidden()
    # Every tmux in the shown rows is highlighted, in titles, breadcrumbs and block text.
    for text, marks in page.locator('#lsg-search-results [role=option]').evaluate_all(f'rows => rows.map(row => [row.innerText, [...row.querySelectorAll("{HIGHLIGHT}")].map(mark => mark.textContent)])'):
        assert marks and all(mark.lower() == 'tmux' for mark in marks) and len(marks) == text.lower().count('tmux'), (text, marks)
    # A block row's breadcrumb starts with its page title.
    for crumb in page.locator('[data-lsg-search-group=blocks] .lsg-search-crumb').all_inner_texts():
        assert crumb.startswith('tmux'), crumb
    # mod+Down shows the whole group and mod+Up five rows again; clicking the name toggles.
    page.keyboard.press('ControlOrMeta+ArrowDown')
    expect(pages).to_have_count(100)
    expect(more).to_contain_text('Show less')
    assert '↑' in more.inner_text()
    page.keyboard.press('ControlOrMeta+ArrowUp')
    expect(pages).to_have_count(5)
    page.locator('[data-lsg-search-group=pages] .lsg-search-group-name').click()
    expect(pages).to_have_count(100)
    expect(field).to_be_focused()
    # An alias row says which page it names.
    complete(page, lambda: field.fill('tmux.conf.local'))
    alias = pages.first
    assert alias.locator('.lsg-search-alias').inner_text() == 'alias of'
    assert alias.inner_text().replace('\n', ' ').split() == ['tmux.conf.local', 'alias', 'of', 'tmux/Config/local'], alias.inner_text()
    assert alias.locator(HIGHLIGHT).first.inner_text() == 'tmux.conf.local'
    # / lists the filters, narrowed by the text after it.
    complete(page, lambda: field.fill('/'))
    assert group_names(page) == ['Filters']
    assert [text.split('\n')[0].replace('— Add filter to search', '').strip() for text in options.all_inner_texts()] == ['Search only pages', 'Search only current page', 'Search only blocks', 'Search only commands']
    complete(page, lambda: field.fill('/block'))
    expect(options).to_have_count(1)
    # Choosing pages removes the /… text, shows the chip and lists every matching page.
    complete(page, lambda: field.fill('/pages'))
    page.keyboard.press('Enter')
    expect(field).to_have_value('')
    expect(page.locator('.lsg-search-filter')).to_be_visible()
    assert page.locator('.lsg-search-filter').inner_text().replace('\n', ' ').startswith('Search only: Pages')
    complete(page, lambda: field.fill('tmux'))
    assert group_names(page) == ['Pages']
    expect(pages).to_have_count(100)
    expect(page.locator('#lsg-search-results [role=option]:not([data-lsg-search-kind=page])')).to_have_count(0)
    expect(page.locator('.lsg-search-more')).to_have_count(0)
    expect(page.locator('.lsg-search-tip')).to_contain_text('to clear search filter')
    # Escape clears the input, then the filter, then closes.
    page.keyboard.press('Escape')
    expect(field).to_have_value('')
    expect(page.locator('.lsg-search-filter')).to_be_visible()
    page.keyboard.press('Escape')
    expect(page.locator('.lsg-search-filter')).to_be_hidden()
    expect(dialog).to_be_visible()
    assert group_names(page) == ['Commands']
    page.keyboard.press('Escape')
    expect(dialog).not_to_be_visible()
    # Backspace on an empty input and the chip's button also remove the filter.
    trigger.click()
    complete(page, lambda: field.fill('/blocks'))
    page.keyboard.press('Enter')
    expect(page.locator('.lsg-search-filter')).to_contain_text('Blocks')
    page.keyboard.press('Backspace')
    expect(page.locator('.lsg-search-filter')).to_be_hidden()
    complete(page, lambda: field.fill('/commands'))
    page.keyboard.press('Enter')
    assert group_names(page) == ['Commands']
    page.get_by_role('button', name='Remove filter').click()
    expect(page.locator('.lsg-search-filter')).to_be_hidden()
    # A click outside the dialog closes it and returns focus.
    page.mouse.click(20, 780)
    expect(dialog).not_to_be_visible()
    expect(trigger).to_be_focused()
    # Keys reach the page again at once after closing, even before the close event.
    trigger.click()
    field.fill('tmux')
    page.keyboard.press('Escape')
    page.keyboard.press('Escape')
    page.keyboard.press('Control+k')
    expect(dialog).to_be_visible()
    page.wait_for_timeout(100)
    expect(dialog).to_be_visible()
    expect(field).to_be_focused()
    page.keyboard.press('Escape')
    expect(dialog).not_to_be_visible()
    # Logseq's look at 1280x800 in the light theme.
    trigger.click()
    look = page.evaluate("""() => {
      const dialog = document.querySelector('#lsg-search-dialog'), input = dialog.querySelector('#lsg-search-input');
      const style = node => getComputedStyle(node);
      return {width: dialog.getBoundingClientRect().width, radius: style(dialog).borderRadius, label: dialog.getAttribute('aria-label'),
        title: dialog.querySelector('h1,h2,h3') === null, size: style(input).fontSize, border: style(input).borderTopWidth,
        placeholder: input.placeholder, name: input.labels[0].textContent, hidden: input.labels[0].getBoundingClientRect().width <= 1,
        body: style(dialog.querySelector('.lsg-search-body')).height};
    }""")
    assert look == {'width': 896, 'radius': '8px', 'label': 'Search', 'title': True, 'size': '20px', 'border': '0px',
                    'placeholder': 'What are you looking for?', 'name': look['name'], 'hidden': True, 'body': '520px'}, look
    assert look['name']
    complete(page, lambda: field.fill('tmux'))
    mark = page.locator(HIGHLIGHT).first.evaluate('node => getComputedStyle(node).backgroundColor')
    assert mark == 'rgb(198, 212, 249)', mark
    page.keyboard.press('Escape')
    page.keyboard.press('Escape')
    # Blocks on the page being viewed move to Current page.
    page.goto(base + '/' + page.evaluate("""async () => {
      const link = document.querySelector('[rel=lsg-search]');
      const manifest = await (await fetch(link.href)).json();
      const titles = await (await fetch(new URL(`lsg-search-titles-${manifest.titles}.json`, link.href))).json();
      return titles.pages.find(page => page[0] === 'tmux/Config/local')[1];
    }"""))
    expect(page.locator('main > h1')).to_have_text('tmux/Config/local')
    trigger.click()
    complete(page, lambda: field.fill('tmux'))
    assert group_names(page) == ['Pages', 'Current page', 'Blocks'], group_names(page)
    current = page.locator('[data-lsg-search-group=current] [role=option]')
    expect(current).to_have_count(1)
    expect(current).to_contain_text('overrides')
    for text in page.locator('[data-lsg-search-group=blocks] [role=option]').all_inner_texts():
        assert 'overrides' not in text, text
    assert errors == [], errors
    context.close()
    print('search proof: grouped dialog complete', flush=True)


def placeholder_rows(browser, base):
    """The result list waits only for shards: block rows draw before their chunk arrives."""
    context = browser.new_context(viewport={'width': 1280, 'height': 800})
    page = context.new_page()
    errors = []
    page.on('pageerror', lambda error: errors.append(str(error)))
    held = []
    page.route('**/lsg-search-chunk-*.json', lambda route: held.append(route))
    page.goto(base + '/index.html')
    page.locator('[data-lsg-search]').click()
    field = page.locator('#lsg-search-input')
    expect(page.locator('#lsg-search-dialog')).to_have_attribute('data-lsg-search-state', 'ready')
    listed = page.evaluate('Number(document.querySelector("#lsg-search-results").dataset.lsgSearchDrawn || 0)')
    completed = page.evaluate('Number(document.querySelector("#lsg-search-results").dataset.lsgSearchComplete || 0)')
    field.fill('needle')
    page.wait_for_function('(before) => Number(document.querySelector("#lsg-search-results").dataset.lsgSearchDrawn) > before', arg=listed)
    rows = page.locator('#lsg-search-results [data-lsg-search-group=blocks] [role=option]')
    expect(rows).to_have_count(3)
    # Each row shows its page title and a placeholder line while its chunk loads.
    for at in range(3):
        expect(rows.nth(at).locator('.lsg-search-crumb')).to_contain_text('Notes')
        expect(rows.nth(at).locator('.lsg-search-placeholder')).to_have_count(1)
    assert held, 'the chunk request is still waiting'
    assert page.evaluate('Number(document.querySelector("#lsg-search-results").dataset.lsgSearchComplete || 0)') == completed
    # The text fills in when the chunk arrives.
    page.route('**/lsg-search-chunk-*.json', lambda route: route.continue_())
    for route in held:
        route.continue_()
    page.wait_for_function('(before) => Number(document.querySelector("#lsg-search-results").dataset.lsgSearchComplete) > before', arg=completed)
    expect(page.locator('#lsg-search-results .lsg-search-placeholder')).to_have_count(0)
    expect(rows.first).to_contain_text('needle')
    assert errors == [], errors
    context.close()
    print('search proof: placeholder rows complete', flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix='lsg-search-browser-') as temp:
        root = Path(temp)
        pages = root / 'garden/pages'
        pages.mkdir(parents=True)
        (pages / 'Notes.md').write_text('- Parent\n  collapsed:: true\n\t- needle child `safe <img src=x onerror=alert(1)>`\n- Other needle result\n- generated placeholder\n  search-exclude:: true\n')
        (pages / 'Notes___Needle.md').write_text('alias:: Find me\n- target needle content\n')
        (pages / 'Notes___Private.md').write_text('public:: false\n- PRIVATE_SENTINEL\n')
        # More than 99 pages and blocks match tmux, so groups show 99+ and Show more.
        (pages / 'tmux.md').write_text('- tmux runs inside a terminal\n- Why tmux\n\t- Sessions survive a closed terminal\n')
        (pages / 'tmux___Config___local.md').write_text('alias:: tmux.conf.local\n- tmux.conf.local overrides tmux.conf\n')
        for number in range(1, 101):
            (pages / f'tmux___Topic {number:03}.md').write_text(f'- Topic {number:03} uses tmux daily\n')
        site = root / 'site'
        subprocess.run([str(args.binary.resolve()), 'publish', '--graph', str(root / 'garden'), '--output', str(site), '--filename-format', 'triple-lowbar', '--include', 'Notes', '--include', 'tmux', '--search'], check=True, capture_output=True)
        nested = site / 'nested/page'
        nested.mkdir(parents=True)
        html = (site / 'index.html').read_text()
        html = html.replace('href="lsg-search-manifest-', 'href="/lsg-search-manifest-').replace('src="lsg-search-ui-', 'src="/lsg-search-ui-').replace('src="lsg-search-start-', 'src="/lsg-search-start-')
        (nested / 'index.html').write_text(html)
        class Quiet(http.server.SimpleHTTPRequestHandler):
            def log_message(self, *_):
                pass
        server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), functools.partial(Quiet, directory=site))
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        base = f'http://127.0.0.1:{server.server_port}'
        try:
            with sync_playwright() as playwright:
                browser = playwright.chromium.launch()
                context = browser.new_context(viewport={'width': 1280, 'height': 800})
                page = context.new_page()
                errors = []
                page.on('pageerror', lambda error: errors.append(str(error)))
                requests = []
                page.on('request', lambda request: requests.append(request.url))
                page.goto(base + '/index.html')
                trigger = page.locator('[data-lsg-search]')
                expect(trigger).to_be_visible()
                assert not any('lsg-search-manifest-' in url or 'lsg-search-engine-' in url or ('lsg-search-' in url and url.endswith('.json')) for url in requests)
                trigger.focus()
                page.keyboard.press('Control+k')
                field = page.locator('#lsg-search-input')
                expect(field).to_be_focused()
                field.fill('needle')
                results = page.locator('#lsg-search-results a')
                options = page.locator('#lsg-search-results [role=option]')
                expect(results).to_have_count(4)
                assert page.locator('#lsg-search-results img').count() == 0
                assert 'safe <img' in page.locator('#lsg-search-results').inner_text()
                assert 'PRIVATE_SENTINEL' not in page.locator('#lsg-search-results').inner_text()
                # One frame evaluates only the newest rapid input; close cancels it.
                completed = field.evaluate('node => { const list=document.querySelector("#lsg-search-results"); const before=Number(list.dataset.lsgSearchComplete); node.value="obsolete"; node.dispatchEvent(new Event("input", {bubbles:true})); node.value="needle"; node.dispatchEvent(new Event("input", {bubbles:true})); return before; }')
                page.wait_for_function('(before) => Number(document.querySelector("#lsg-search-results").dataset.lsgSearchComplete) > before', arg=completed)
                assert page.locator('#lsg-search-results').get_attribute('data-lsg-search-complete') == str(completed + 1)
                expect(results).to_have_count(4)
                                # IME candidates own composing keys, including legacy keyCode229.
                field.focus()
                composition = field.evaluate('''node => {
                  node.dispatchEvent(new CompositionEvent('compositionstart', {bubbles: true, data: 'に'}));
                  const cases = [
                    {key: 'Escape', isComposing: true},
                    {key: 'ArrowDown', isComposing: true},
                    {key: 'ArrowUp', keyCode: 229},
                    {key: 'Enter', isComposing: true}
                  ];
                  const outcomes = cases.map(options => {
                    const event = new KeyboardEvent('keydown', {bubbles: true, cancelable: true, ...options});
                    node.dispatchEvent(event);
                    return {prevented: event.defaultPrevented, focused: document.activeElement === node, open: document.querySelector('#lsg-search-dialog').open};
                  });
                  node.dispatchEvent(new CompositionEvent('compositionend', {bubbles: true, data: 'に'}));
                  return outcomes;
                }''')
                assert all(not state['prevented'] and state['focused'] and state['open'] for state in composition), composition
                # The input keeps focus; the highlight moves through the rows.
                expect(options.nth(0)).to_have_attribute('aria-selected', 'true')
                page.keyboard.press('ArrowDown')
                expect(options.nth(1)).to_have_attribute('aria-selected', 'true')
                expect(field).to_have_attribute('aria-activedescendant', options.nth(1).get_attribute('id'))
                page.keyboard.press('ArrowUp')
                expect(options.nth(0)).to_have_attribute('aria-selected', 'true')
                expect(field).to_be_focused()
                page.keyboard.press('g')
                page.keyboard.press('a')
                expect(page.locator('#lsg-search-dialog')).to_be_visible()
                expect(field).to_have_value('needlega')
                page.keyboard.press('Control+k')
                expect(field).to_be_focused()
                page.keyboard.press('Escape')
                expect(field).to_have_value('')
                expect(page.locator('#lsg-search-dialog')).to_be_visible()
                page.keyboard.press('Escape')
                expect(page.locator('#lsg-search-dialog')).not_to_be_visible()
                expect(trigger).to_be_focused()
                before = len(requests)
                trigger.click()
                field.fill('needle')
                expect(results).to_have_count(4)
                assert not any('lsg-search-' in url for url in requests[before:]), 'cached reopen fetched data'
                field.fill('no-match-at-all')
                expect(results).to_have_count(0)
                expect(page.locator('#lsg-search-status')).to_have_text('No results.')
                field.fill('needle')
                expect(results).to_have_count(4)
                # Shared appearance variables follow dark/system and mobile geometry.
                page.evaluate("window.logseqGardenerTheme.set('dark')")
                dark = page.locator('#lsg-search-dialog').evaluate('node => getComputedStyle(node).backgroundColor')
                page.evaluate("window.logseqGardenerTheme.set('light')")
                light = page.locator('#lsg-search-dialog').evaluate('node => getComputedStyle(node).backgroundColor')
                assert light != dark
                page.emulate_media(color_scheme='dark')
                page.evaluate("window.logseqGardenerTheme.set('system')")
                assert page.locator('#lsg-search-dialog').evaluate('node => getComputedStyle(node).backgroundColor') == dark
                # Native Tab stays within the modal.
                field.focus()
                page.keyboard.press('Tab')
                expect(page.locator('.lsg-search-hint')).to_be_focused()
                page.keyboard.press('Shift+Tab')
                expect(field).to_be_focused()
                page.set_viewport_size({'width': 390, 'height': 844})
                bounds = page.locator('#lsg-search-dialog').bounding_box()
                assert bounds['x'] >= 0 and bounds['x'] + bounds['width'] <= 390
                # Modified activation keeps the dialog open.
                field.focus()
                with context.expect_page() as popup_info:
                    results.nth(0).click(modifiers=['ControlOrMeta'])
                popup_info.value.close()
                expect(page.locator('#lsg-search-dialog')).to_be_visible()
                # Host delegates ordinary links after the controller closes, with persistent state.
                page.evaluate('''() => {
                  window.persistedState = {playhead: 123};
                  window.logseqGardenerInitSearch(document, {resolvePage: (_title, href) => new URL(href, location.href).href});
                  window.logseqGardenerInitSearch();
                  document.addEventListener('click', async event => {
                    const link = event.target.closest('#lsg-search-results a');
                    if (!link || event.ctrlKey || event.metaKey) return;
                    event.preventDefault();
                    window.wasClosedBeforeHost = !document.querySelector('#lsg-search-dialog').open;
                    const html = await (await fetch(link.href)).text();
                    const incoming = new DOMParser().parseFromString(html, 'text/html');
                    document.querySelector('main').replaceWith(incoming.querySelector('main'));
                    document.head.querySelector('[rel=lsg-search]').replaceWith(incoming.head.querySelector('[rel=lsg-search]'));
                    window.logseqGardenerInitSearch(document);
                    window.hostDone = true;
                  });
                }''')
                field.focus()
                page.keyboard.press('Enter')
                page.wait_for_function('window.hostDone')
                assert page.evaluate('window.wasClosedBeforeHost && window.persistedState.playhead === 123')
                assert page.locator('#lsg-search-dialog').count() == 1
                trigger.click()
                expect(field).to_be_focused()
                print('search proof: SPA complete', flush=True)
                # A changed identity closes the dialog and cannot reuse old results.
                page.evaluate("""const metadata=document.head.querySelector('[rel=lsg-search]'); metadata.href=new URL('lsg-search-manifest-fedcba9876543210.json', metadata.href); metadata.dataset.revision='fedcba9876543210'; window.logseqGardenerInitSearch()""")
                expect(page.locator('#lsg-search-dialog')).not_to_be_visible()
                trigger.click()
                expect(page.locator('#lsg-search-status')).to_contain_text('unavailable')
                expect(page.locator('#lsg-search-results a')).to_have_count(0)
                # Missing identity hides unusable controls.
                page.evaluate("document.head.querySelector('[rel=lsg-search]').remove(); window.logseqGardenerInitSearch()")
                expect(trigger).not_to_be_visible()
                expect(page.locator('#lsg-search-dialog')).not_to_be_visible()
                assert errors == [], errors
                context.close()
                # The default result route uses the manifest directory from a nested page.
                context = browser.new_context()
                page = context.new_page()
                page.goto(base + '/nested/page/index.html')
                page.locator('[data-lsg-search]').click()
                page.locator('#lsg-search-input').fill('needle')
                expect(page.locator('#lsg-search-results a')).to_have_count(4)
                destination = page.locator('#lsg-search-results a').first.get_attribute('href')
                assert destination.startswith(base + '/p-') and '/nested/' not in destination
                context.close()
                # A failed load is actionable, never an empty successful search or automatic reload.
                context = browser.new_context()
                page = context.new_page()
                page.route('**/lsg-search-manifest-*.json', lambda route: route.fulfill(status=404, body='missing'))
                page.goto(base + '/index.html')
                page.locator('[data-lsg-search]').click()
                expect(page.locator('#lsg-search-status')).to_contain_text('unavailable')
                expect(page.get_by_role('button', name='Retry search')).to_be_visible()
                page.unroute('**/lsg-search-manifest-*.json')
                page.get_by_role('button', name='Retry search').click()
                page.locator('#lsg-search-input').fill('needle')
                expect(page.locator('#lsg-search-results a')).to_have_count(4)
                context.close()
                print('search proof: retry complete', flush=True)
                # Corrupt corpus and offline opens stay actionable and recover on retry.
                context = browser.new_context()
                page = context.new_page()
                page.route('**/lsg-search-*.json', lambda route: route.continue_() if 'manifest-' in route.request.url else route.fulfill(status=200, body='{broken'))
                page.goto(base + '/index.html')
                page.locator('[data-lsg-search]').click()
                expect(page.locator('#lsg-search-status')).to_contain_text('unavailable')
                expect(page.locator('#lsg-search-results a')).to_have_count(0)
                page.unroute('**/lsg-search-*.json')
                page.get_by_role('button', name='Retry search').click()
                expect(page.locator('#lsg-search-dialog')).to_have_attribute('data-lsg-search-state', 'ready')
                context.close()
                context = browser.new_context()
                page = context.new_page()
                page.goto(base + '/index.html')
                page.locator('[data-lsg-search]').click()
                expect(page.locator('#lsg-search-dialog')).to_have_attribute('data-lsg-search-state', 'ready')
                page.keyboard.press('Escape')
                page.evaluate("const link=document.head.querySelector('[rel=lsg-search]');link.remove();window.logseqGardenerInitSearch();document.head.append(link);window.logseqGardenerInitSearch()")
                context.set_offline(True)
                page.locator('[data-lsg-search]').click()
                expect(page.locator('#lsg-search-status')).to_contain_text('unavailable')
                context.set_offline(False)
                page.get_by_role('button', name='Retry search').click()
                expect(page.locator('#lsg-search-dialog')).to_have_attribute('data-lsg-search-state', 'ready')
                context.close()
                # Closing during a deferred first fetch cannot publish stale results.
                context = browser.new_context()
                page = context.new_page()
                page.goto(base + '/index.html')
                page.evaluate("""() => { window.originalFetch=window.fetch; window.fetch=(...args)=>new Promise(resolve=>{window.releaseSearch=()=>resolve(window.originalFetch(...args));}); }""")
                page.locator('[data-lsg-search]').click()
                page.wait_for_function('Boolean(window.releaseSearch)')
                page.keyboard.press('Escape')
                page.evaluate('window.fetch=window.originalFetch; window.releaseSearch()')
                page.locator('[data-lsg-search]').click()
                page.locator('#lsg-search-input').fill('needle')
                expect(page.locator('#lsg-search-results a')).to_have_count(4)
                expect(page.locator('#lsg-search-status')).not_to_contain_text('unavailable')
                context.close()
                print('search proof: stale close complete', flush=True)
                # Missing post-paint starter leaves the page usable and explains recovery.
                context = browser.new_context()
                page = context.new_page()
                page.route('**/lsg-search-start-*.mjs', lambda route: route.fulfill(status=404, body='missing'))
                page.goto(base + '/index.html')
                expect(page.get_by_role('status')).to_contain_text('Reload this page to retry search')
                expect(page.locator('main h1')).to_be_visible()
                expect(page.locator('[data-lsg-search]')).not_to_be_visible()
                assert page.url == base + '/index.html'
                context.close()
                # Host init before and during the post-paint starter import survives handoff.
                context = browser.new_context()
                context.add_init_script('''() => {
                  let api, first = true;
                  Object.defineProperty(window, 'logseqGardenerInitSearch', {configurable:true,
                    get: () => api, set: value => {api=value;if(first){first=false;value(document,{resolvePage:()=>'/before-starter/'});}}
                  });
                }''')
                page = context.new_page()
                starter = []
                page.route('**/lsg-search-start-*.mjs', lambda route: starter.append(route))
                page.goto(base + '/index.html')
                page.wait_for_timeout(100)
                assert starter
                expect(page.locator('[data-lsg-search]')).not_to_be_visible()
                page.evaluate("window.logseqGardenerInitSearch(document,{resolvePage:()=>'/during-starter/'})")
                starter.pop().continue_()
                expect(page.locator('[data-lsg-search]')).to_be_visible()
                page.locator('[data-lsg-search]').click()
                page.locator('#lsg-search-input').fill('needle')
                expect(page.locator('#lsg-search-results a')).to_have_count(4)
                assert page.locator('#lsg-search-results a').first.get_attribute('href') == '/during-starter/'
                context.close()
                # Bootstrap module failure remains retryable; latest host options survive import.
                context = browser.new_context()
                page = context.new_page()
                page.route('**/lsg-search-ui-*.mjs*', lambda route: route.fulfill(status=404, body='missing'))
                page.goto(base + '/index.html')
                page.locator('[data-lsg-search]').click()
                expect(page.get_by_role('status')).to_contain_text('could not load')
                page.unroute('**/lsg-search-ui-*.mjs*')
                held = []
                page.route('**/lsg-search-ui-*.mjs*', lambda route: held.append(route))
                page.locator('[data-lsg-search]').click()
                page.wait_for_timeout(100)
                assert held, 'module retry did not request the controller'
                page.keyboard.type('gatt')
                page.evaluate("for(let i=0;i<600;i++)document.dispatchEvent(new KeyboardEvent('keydown',{key:'x',bubbles:true,cancelable:true}))")
                assert page.url == base + '/index.html'
                assert page.evaluate('window.logseqGardenerTheme.choice()') == 'system'
                page.evaluate("window.logseqGardenerInitSearch(document,{resolvePage:()=>'/readable-result/'})")
                held.pop().continue_()
                expect(page.locator('#lsg-search-input')).to_be_focused()
                expect(page.locator('#lsg-search-input')).to_have_value('gatt' + 'x' * 508)
                page.locator('#lsg-search-input').fill('needle')
                expect(page.locator('#lsg-search-results a')).to_have_count(4)
                assert page.locator('#lsg-search-results a').first.get_attribute('href') == '/readable-result/'
                context.close()
                # Offline before the first UI import is actionable; reopening with fetched
                # shards and chunks stays offline.
                context = browser.new_context()
                page = context.new_page()
                page.goto(base + '/index.html')
                expect(page.locator('[data-lsg-search]')).to_be_visible()
                context.set_offline(True)
                page.locator('[data-lsg-search]').click()
                expect(page.get_by_role('status')).to_contain_text('could not load')
                expect(page.locator('main h1')).to_be_visible()
                context.set_offline(False)
                page.locator('[data-lsg-search]').click()
                expect(page.locator('#lsg-search-dialog')).to_have_attribute('data-lsg-search-state', 'ready')
                page.locator('#lsg-search-input').fill('needle')
                expect(page.locator('#lsg-search-results a')).to_have_count(4)
                page.keyboard.press('Escape')
                page.keyboard.press('Escape')
                context.set_offline(True)
                page.locator('[data-lsg-search]').click()
                page.locator('#lsg-search-input').fill('needle')
                expect(page.locator('#lsg-search-results a')).to_have_count(4)
                context.close()
                # Escape restores ordinary page typing/Tab while the module remains held.
                context = browser.new_context()
                page = context.new_page()
                held = []
                page.route('**/lsg-search-ui-*.mjs*', lambda route: held.append(route))
                page.goto(base + '/index.html')
                page.evaluate("const input=document.createElement('input');input.id='page-input';const next=document.createElement('button');next.id='page-next';next.textContent='Next';document.body.append(input,next)")
                page.locator('#page-input').focus()
                page.keyboard.press('Control+k')
                page.wait_for_timeout(100)
                assert held
                page.keyboard.press('Escape')
                page.keyboard.type('ordinary')
                expect(page.locator('#page-input')).to_have_value('ordinary')
                page.keyboard.press('Tab')
                expect(page.locator('#page-next')).to_be_focused()
                held.pop().continue_()
                page.wait_for_timeout(100)
                assert page.locator('#lsg-search-dialog').count() == 0
                page.locator('[data-lsg-search]').click()
                expect(page.locator('#lsg-search-input')).to_be_focused()
                page.keyboard.press('Escape')
                expect(page.locator('[data-lsg-search]')).to_be_focused()
                context.close()
                # A missing publication during module loading cannot be re-enabled by handoff.
                context = browser.new_context()
                page = context.new_page()
                held = []
                page.route('**/lsg-search-ui-*.mjs*', lambda route: held.append(route))
                page.goto(base + '/index.html')
                page.locator('[data-lsg-search]').click()
                page.wait_for_timeout(100)
                assert held
                page.evaluate("document.head.querySelector('[rel=lsg-search]').remove();window.logseqGardenerInitSearch()")
                held.pop().continue_()
                page.wait_for_timeout(100)
                expect(page.locator('[data-lsg-search]')).not_to_be_visible()
                assert page.locator('#lsg-search-dialog').count() == 0
                context.close()
                nojs = browser.new_context(java_script_enabled=False)
                page = nojs.new_page()
                page.goto(base + '/index.html')
                expect(page.locator('[data-lsg-search]')).not_to_be_visible()
                expect(page.locator('[data-lsg-all-pages]')).to_have_attribute('href', 'index.html')
                nojs.close()
                grouped_dialog(browser, base)
                placeholder_rows(browser, base)
                browser.close()
            print(json.dumps({'search_browser': 'passed', 'keyboard': True, 'mobile_themes': True, 'lazy_cache': True, 'spa_persistence': True, 'failure_retry': True, 'no_javascript': True, 'grouped_dialog': True, 'placeholder_rows': True}))
        finally:
            server.shutdown()
            server.server_close()


if __name__ == '__main__':
    main()
