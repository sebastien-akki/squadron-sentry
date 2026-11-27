const me = Math.random().toString(36).slice(2)
const lang = (navigator.language || 'en').toLowerCase()
const fr = lang.startsWith('fr')

const T = fr
  ? {
      refresh: 'Actualiser',
      loading: 'Chargement…',
      noAccount: 'Ajoutez votre organisation Sentry dans Réglages › Extensions › Sentry : serveur, slug de l’organisation et jeton personnel.',
      choose: 'Choisir les projets Sentry',
      done: 'Terminé',
      following: 'Suivis pour ce projet :',
      noFollow: 'Aucun projet Sentry suivi pour ce projet Squadron. Choisissez-les : les erreurs arrivées ensuite seront notifiées et triées.',
      noIssue: 'Aucune erreur non résolue ces 14 derniers jours.',
      events: (n) => `${n} événement${n > 1 ? 's' : ''}`,
      users: (n) => `${n} utilisateur${n > 1 ? 's' : ''}`,
      seen: 'vue',
      regressed: 'revenue',
      ready: 'plan proposé',
      running: 'analyse en cours…',
      failed: 'analyse en échec',
      back: '← Erreurs',
      fix: 'Corriger avec un agent',
      propose: 'Analyser et planifier',
      repropose: 'Relancer l’analyse',
      resolve: 'Résoudre',
      ignore: 'Ignorer',
      stack: 'Stack trace et contexte envoyés à Claude',
      noProposal: 'Pas encore d’analyse. Claude Code lira le dépôt sans le modifier, puis proposera un diagnostic et un plan d’exécution.',
      proposing: 'Claude Code lit le code et prépare son plan (jusqu’à 2 minutes)…',
      failedText: 'L’analyse n’a pas abouti :',
      opened: 'La fenêtre Nouvel agent est ouverte, avec l’erreur en contexte. Une fois lancé, l’agent présentera son plan et attendra votre accord avant de toucher au code.',
      auto: 'Chaque erreur nouvelle ou revenue est analysée automatiquement, sauf celles que le tri met de côté.',
      manual: 'Analyse automatique coupée : ouvrez une erreur et cliquez sur Analyser et planifier.',
      triage: 'Tri',
      sortedAt: (w) => `par Haiku, ${w}`,
      notSorted: 'pas encore fait',
      sorting: 'Haiku trie les erreurs…',
      resort: 'Relancer le tri',
      triageFailed: 'Le tri n’a pas abouti :',
      nothingSorted: 'Rien à regrouper ni à mettre de côté.',
      same: 'Même sujet',
      merge: 'Fusionner dans Sentry',
      confirmMerge: 'Confirmer la fusion',
      noise: 'Sans raison d’être traitées',
      reasons: { mep: 'MEP', local: 'local', test: 'test', other: 'autre' },
      noiseNote: (r, why) => `Le tri la met de côté (${r}) : ${why}`,
      closeAll: 'Tout résoudre',
      confirmClose: 'Confirmer',
      staleLabel: 'Sans occurrence depuis',
      staleDays: 'jours',
      list: 'Lister',
      staleNone: (d) => `Aucune erreur non résolue sans occurrence depuis ${d} jours.`,
      staleCount: (n, d) => `${n} erreur${n > 1 ? 's' : ''} sans occurrence depuis ${d} jours :`,
      total: (n) => `${n} erreur${n > 1 ? 's' : ''} non résolue${n > 1 ? 's' : ''} sur 14 jours`,
      shown: (n) => `les ${n} premières affichées`,
      sortBy: 'Trier par',
      sorts: { date: 'Dernière occurrence', new: 'Plus récentes', freq: 'Plus d’occurrences', user: 'Plus d’utilisateurs' },
      colError: 'Erreur',
      colCount: 'Occurrences',
      colAge: 'Âge',
      ageHint: (d) => `Première occurrence : ${d}`,
      selectAll: 'Tout sélectionner',
      select: 'Sélectionner',
      selected: (n) => `${n} sélectionnée${n > 1 ? 's' : ''}`,
      clear: 'Désélectionner',
      mergeNeeds: 'Fusionner demande au moins deux erreurs du même projet Sentry.',
      check: 'Cocher ces erreurs',
      units: { min: 'min', h: 'h', d: 'j' },
      stats: { count: 'Occurrences', users: 'Utilisateurs', first: 'Première occurrence', last: 'Dernière occurrence', level: 'Niveau', project: 'Projet', status: 'Statut' },
      substatus: { new: 'nouvelle', regressed: 'revenue', ongoing: 'en cours', escalating: 'en hausse', archived_until_escalating: 'archivée' },
    }
  : {
      refresh: 'Refresh',
      loading: 'Loading…',
      noAccount: 'Add your Sentry organization in Settings › Extensions › Sentry: server, organization slug and personal token.',
      choose: 'Choose Sentry projects',
      done: 'Done',
      following: 'Followed for this project:',
      noFollow: 'No Sentry project is followed for this Squadron project. Choose them: errors that come in afterwards will be notified and sorted.',
      noIssue: 'No unresolved error in the last 14 days.',
      events: (n) => `${n} event${n > 1 ? 's' : ''}`,
      users: (n) => `${n} user${n > 1 ? 's' : ''}`,
      seen: 'seen',
      regressed: 'back',
      ready: 'plan proposed',
      running: 'analysing…',
      failed: 'analysis failed',
      back: '← Errors',
      fix: 'Fix with an agent',
      propose: 'Analyse and plan',
      repropose: 'Analyse again',
      resolve: 'Resolve',
      ignore: 'Ignore',
      stack: 'Stack trace and context sent to Claude',
      noProposal: 'No analysis yet. Claude Code will read the repository without changing it, then propose a diagnosis and a plan.',
      proposing: 'Claude Code is reading the code and preparing its plan (up to 2 minutes)…',
      failedText: 'The analysis did not finish:',
      opened: 'The New agent window is open, with the error as context. Once started, the agent will show its plan and wait for your go-ahead before touching the code.',
      auto: 'Each new or returning error is analysed automatically, except those the triage sets aside.',
      manual: 'Automatic analysis is off: open an error and click Analyse and plan.',
      triage: 'Triage',
      sortedAt: (w) => `by Haiku, ${w}`,
      notSorted: 'not done yet',
      sorting: 'Haiku is sorting the errors…',
      resort: 'Sort again',
      triageFailed: 'The triage did not finish:',
      nothingSorted: 'Nothing to group or set aside.',
      same: 'Same subject',
      merge: 'Merge in Sentry',
      confirmMerge: 'Confirm the merge',
      noise: 'Nothing to fix',
      reasons: { mep: 'deployment', local: 'local', test: 'test', other: 'other' },
      noiseNote: (r, why) => `The triage sets it aside (${r}): ${why}`,
      closeAll: 'Resolve all',
      confirmClose: 'Confirm',
      staleLabel: 'Not seen for',
      staleDays: 'days',
      list: 'List',
      staleNone: (d) => `No unresolved error left unseen for ${d} days.`,
      staleCount: (n, d) => `${n} error${n > 1 ? 's' : ''} not seen for ${d} days:`,
      total: (n) => `${n} unresolved error${n > 1 ? 's' : ''} over 14 days`,
      shown: (n) => `the first ${n} shown`,
      sortBy: 'Sort by',
      sorts: { date: 'Last seen', new: 'Newest', freq: 'Most events', user: 'Most users' },
      colError: 'Error',
      colCount: 'Events',
      colAge: 'Age',
      ageHint: (d) => `First seen: ${d}`,
      selectAll: 'Select all',
      select: 'Select',
      selected: (n) => `${n} selected`,
      clear: 'Clear selection',
      mergeNeeds: 'Merging needs at least two errors of the same Sentry project.',
      check: 'Tick these errors',
      units: { min: 'min', h: 'h', d: 'd' },
      stats: { count: 'Events', users: 'Users', first: 'First seen', last: 'Last seen', level: 'Level', project: 'Project', status: 'Status' },
      substatus: { new: 'new', regressed: 'back', ongoing: 'ongoing', escalating: 'escalating', archived_until_escalating: 'archived' },
    }

const state = { data: null, projects: null, picking: false, open: null, context: {}, error: '', note: '', busy: false, sorting: false, confirm: '', days: null, stale: null, staleBusy: false, selected: new Set(), sort: 'date' }
const send = (type, extra = {}) => squadron.send({ ...extra, type, to: me, lang, sort: state.sort })

const relative = new Intl.RelativeTimeFormat(lang, { numeric: 'auto' })
const number = new Intl.NumberFormat(lang)
const dateTime = new Intl.DateTimeFormat(lang, { dateStyle: 'medium', timeStyle: 'short' })
const fullDate = (iso) => (Date.parse(iso) ? dateTime.format(new Date(iso)) : '')

function age(iso) {
  if (!Date.parse(iso)) return ''
  const minutes = Math.max(0, Math.round((Date.now() - Date.parse(iso)) / 60000))
  if (minutes < 60) return `${minutes} ${T.units.min}`
  if (minutes < 2880) return `${Math.round(minutes / 60)} ${T.units.h}`
  return `${Math.round(minutes / 1440)} ${T.units.d}`
}
function when(iso) {
  const at = Date.parse(iso)
  if (!at) return ''
  const minutes = Math.round((at - Date.now()) / 60000)
  if (minutes > -1) return relative.format(0, 'minute')
  if (minutes > -60) return relative.format(minutes, 'minute')
  if (minutes > -1440) return relative.format(Math.round(minutes / 60), 'hour')
  return relative.format(Math.round(minutes / 1440), 'day')
}

function h(tag, props = {}, ...children) {
  const el = document.createElement(tag)
  for (const [k, v] of Object.entries(props)) {
    if (v === undefined || v === null || v === false) continue
    if (k.startsWith('on')) el.addEventListener(k.slice(2), v)
    else if (k in el) el[k] = v
    else el.setAttribute(k, v)
  }
  el.append(...children.flat(Infinity).filter((c) => c !== null && c !== undefined && c !== false))
  return el
}

const found = () => state.open && (state.data?.issues || []).find((x) => x.account === state.open.account && x.issue.id === state.open.id)

const inline = (text) =>
  text
    .split(/(`[^`]+`|\*\*[^*]+\*\*)/)
    .filter(Boolean)
    .map((p) => (p.length > 2 && p[0] === '`' && p.endsWith('`') ? h('code', {}, p.slice(1, -1)) : p.length > 4 && p.startsWith('**') && p.endsWith('**') ? h('strong', {}, p.slice(2, -2)) : p))

// Headings, paragraphs and fenced code; diff lines coloured. Text only, never HTML.
function proposalView(text) {
  const out = []
  const lines = text.split('\n')
  for (let i = 0; i < lines.length; i++) {
    const l = lines[i]
    if (/^\s*```/.test(l)) {
      const code = []
      while (++i < lines.length && !/^\s*```/.test(lines[i])) code.push(lines[i])
      out.push(h('pre', {}, code.map((c, k) => [h('span', { className: /^\+(?!\+\+)/.test(c) ? 'add' : /^-(?!--)/.test(c) ? 'del' : '' }, c), k < code.length - 1 ? '\n' : ''])))
    } else if (/^#{1,6}\s/.test(l)) out.push(h('h3', {}, l.replace(/^#+\s*/, '')))
    else if (l.trim()) out.push(h('p', {}, inline(l)))
  }
  return h('div', { className: 'proposal' }, out)
}

function tag(p) {
  if (!p) return null
  if (p.status === 'ready') return h('span', { className: 'tag ok' }, T.ready)
  if (p.status === 'running') return h('span', { className: 'tag' }, T.running)
  return h('span', { className: 'tag err' }, T.failed)
}

const noiseOf = (account, id) => (state.data?.triage?.noise || []).find((n) => n.account === account && n.id === id)
const accountsOf = (items) => [...new Set(items.map((x) => x.account))].map((a) => [a, items.filter((x) => x.account === a).map((x) => x.id)])

// A second click within 4 s confirms: these change Sentry for the whole team.
function confirmButton(id, label, again, run, off = false) {
  const armed = state.confirm === id
  const click = () => {
    if (!armed) {
      state.confirm = id
      setTimeout(() => {
        if (state.confirm === id) {
          state.confirm = ''
          render()
        }
      }, 4000)
      return render()
    }
    state.confirm = ''
    state.busy = true
    run()
    render()
  }
  return h('button', { className: armed ? 'danger' : '', disabled: state.busy || off, onclick: click }, armed ? again : label)
}

function setStatus(account, ids, status = 'resolved') {
  state.busy = true
  send('status', { account, ids, status })
  ids.forEach((id) => state.selected.delete(`${account}|${id}`))
  if (state.stale) state.stale.issues = state.stale.issues.filter((x) => !(x.account === account && ids.includes(x.issue.id)))
  render()
}

function staleView() {
  const s = state.stale
  const run = () => {
    state.staleBusy = true
    send('stale', { days: Number(state.days) || 30 })
    render()
  }
  const items = s ? s.issues.map((x) => ({ account: x.account, id: x.issue.id })) : []
  return [
    h(
      'div',
      { className: 'bar' },
      h('label', { className: 'days' }, T.staleLabel, ' ', h('input', { type: 'number', min: 1, max: 89, value: state.days ?? 30, oninput: (e) => (state.days = e.target.value), onkeydown: (e) => e.key === 'Enter' && run() }), ' ', T.staleDays),
      h('button', { disabled: state.staleBusy, onclick: run }, T.list),
    ),
    s &&
      (s.issues.length
        ? [
            h('p', { className: 'muted' }, T.staleCount(s.issues.length, s.days)),
            s.issues.map(({ account, issue: i }) =>
              h('div', { className: 'row' }, h('span', { className: 'what' }, h('span', { className: 'key' }, i.short_id), ' ', i.title, h('span', { className: 'muted' }, ` · ${T.seen} ${when(i.last_seen)}`)), h('button', { disabled: state.busy, onclick: () => setStatus(account, [i.id]) }, T.resolve)),
            ),
            h(
              'div',
              { className: 'bar end' },
              confirmButton('stale', T.closeAll, T.confirmClose, () => accountsOf(items).forEach(([a, ids]) => setStatus(a, ids))),
            ),
          ]
        : h('p', { className: 'muted' }, T.staleNone(s.days))),
  ]
}

function triageView() {
  const d = state.data
  const t = d.triage || { at: 0, groups: [], noise: [], errors: [] }
  const key = (account, id) => {
    const x = d.issues.find((y) => y.account === account && y.issue.id === id)
    return x && h('button', { className: 'link', onclick: () => openIssue(account, id) }, x.issue.short_id)
  }
  const sort = () => {
    state.sorting = true
    send('triage')
    render()
  }
  return h(
    'section',
    { className: 'triage' },
    d.issues.length > 0 && [
      h('div', { className: 'bar' }, h('strong', {}, T.triage), h('span', { className: 'status muted' }, state.sorting ? T.sorting : t.at > 0 ? T.sortedAt(when(new Date(t.at).toISOString())) : T.notSorted), h('button', { disabled: state.sorting || state.busy, onclick: sort }, T.resort)),
      t.errors.map((e) => h('p', { className: 'error' }, `${T.triageFailed} ${e}`)),
      t.at > 0 && !t.groups.length && !t.noise.length && h('p', { className: 'muted' }, T.nothingSorted),
      t.groups.length > 0 && [
        h('h3', {}, T.same),
        t.groups.map((g) =>
          h(
            'div',
            { className: 'row' },
            h('span', { className: 'what' }, h('strong', {}, g.subject), ' · ', g.ids.map((id) => [key(g.account, id), ' '])),
            h('button', { onclick: () => { g.ids.forEach((id) => state.selected.add(`${g.account}|${id}`)); render() } }, T.check),
          ),
        ),
      ],
      t.noise.length > 0 && [
        h('h3', {}, T.noise),
        t.noise.map((n) =>
          h(
            'div',
            { className: 'row' },
            h('span', { className: 'what' }, key(n.account, n.id), ' ', h('span', { className: 'tag' }, T.reasons[n.reason] || n.reason), ' ', n.why),
            h('button', { disabled: state.busy, onclick: () => setStatus(n.account, [n.id]) }, T.resolve),
            h('button', { disabled: state.busy, onclick: () => setStatus(n.account, [n.id], 'ignored') }, T.ignore),
          ),
        ),
        h('div', { className: 'bar end' }, confirmButton('noise', T.closeAll, T.confirmClose, () => accountsOf(t.noise).forEach(([a, ids]) => setStatus(a, ids)))),
      ],
    ],
    staleView(),
  )
}

function openIssue(account, id) {
  state.open = { account, id }
  state.note = ''
  if (!state.context[id]) send('detail', { account, id })
  render()
}

const keyOf = (x) => `${x.account}|${x.issue.id}`

function bulkBar(picked) {
  const items = picked.map((x) => ({ account: x.account, id: x.issue.id }))
  const all = (status) => accountsOf(items).forEach(([a, ids]) => setStatus(a, ids, status))
  const one = picked[0]
  const mergeable = picked.length >= 2 && picked.every((x) => x.account === one.account && x.issue.project_id === one.issue.project_id)
  const merge = () => {
    state.selected.clear()
    send('merge', { account: one.account, project: one.issue.project_id, ids: picked.map((x) => x.issue.id) })
  }
  return h(
    'div',
    { className: 'bar bulk', role: 'region', 'aria-label': T.selected(picked.length) },
    h('strong', { className: 'status' }, T.selected(picked.length)),
    h('button', { disabled: state.busy, onclick: () => all('resolved') }, T.resolve),
    h('button', { disabled: state.busy, onclick: () => all('ignored') }, T.ignore),
    h('span', { title: mergeable ? '' : T.mergeNeeds }, confirmButton(`merge:${picked.map(keyOf).join()}`, T.merge, T.confirmMerge, merge, !mergeable)),
    h('button', { onclick: () => { state.selected.clear(); render() } }, T.clear),
  )
}

function issuesView() {
  const d = state.data
  const picked = d.issues.filter((x) => state.selected.has(keyOf(x)))
  const allOn = picked.length === d.issues.length
  const toggleAll = () => {
    if (allOn) state.selected.clear()
    else d.issues.forEach((x) => state.selected.add(keyOf(x)))
    render()
  }
  const sortSelect = h(
    'label',
    { className: 'sort muted' },
    T.sortBy,
    ' ',
    h('select', { onchange: (e) => { state.sort = e.target.value; state.data = null; send('refresh'); render() } }, Object.entries(T.sorts).map(([v, l]) => h('option', { value: v, selected: v === state.sort }, l))),
  )
  const total = d.total ?? d.issues.length
  return [
    h('div', { className: 'bar' }, h('strong', { className: 'count' }, T.total(total), total > d.issues.length && h('span', { className: 'muted' }, ` · ${T.shown(d.issues.length)}`)), sortSelect),
    picked.length > 0 && bulkBar(picked),
    h(
      'ul',
      { className: 'issues' },
      h('li', { className: 'head muted' }, h('input', { type: 'checkbox', checked: allOn, 'aria-label': T.selectAll, onchange: toggleAll }), h('span', {}, T.colError), h('span', { className: 'num' }, T.colCount), h('span', { className: 'num' }, T.colAge), h('span', {})),
      d.issues.map((x) => {
        const { account, issue: i, proposal } = x
        const n = noiseOf(account, i.id)
        const on = state.selected.has(keyOf(x))
        const pick = (e) => {
          if (e.target.checked) state.selected.add(keyOf(x))
          else state.selected.delete(keyOf(x))
          render()
        }
        return h(
          'li',
          { className: on ? 'picked' : '' },
          h('input', { type: 'checkbox', checked: on, 'aria-label': `${T.select} ${i.short_id}`, onchange: pick }),
          h(
            'button',
            { className: 'issue', title: `${i.short_id} · ${i.title}`, onclick: () => openIssue(account, i.id) },
            h('span', { className: 'line' }, h('span', { className: 'key' }, i.short_id), h('span', { className: 'title' }, i.title)),
            h(
              'span',
              { className: 'meta muted' },
              [i.project, T.users(i.users), `${T.seen} ${when(i.last_seen)}`].join(' · '),
              i.substatus === 'regressed' && [' ', h('span', { className: 'tag err' }, T.regressed)],
              n && [' ', h('span', { className: 'tag' }, T.reasons[n.reason] || n.reason)],
              proposal && [' ', tag(proposal)],
            ),
          ),
          h('span', { className: 'num strong' }, number.format(Number(i.count) || 0)),
          h('span', { className: 'num', title: T.ageHint(fullDate(i.first_seen)) }, age(i.first_seen)),
          h(
            'span',
            { className: 'acts' },
            h('button', { className: 'small', disabled: state.busy, onclick: () => setStatus(account, [i.id]) }, T.resolve),
            h('button', { className: 'small', disabled: state.busy, onclick: () => setStatus(account, [i.id], 'ignored') }, T.ignore),
          ),
        )
      }),
    ),
  ]
}

function listView() {
  const d = state.data
  if (!d.accounts.length) return [h('p', { className: 'muted empty' }, T.noAccount)]
  const followed = h('div', { className: 'follows' }, d.follows.length ? [h('span', { className: 'muted' }, T.following), d.follows.map((f) => h('span', { className: 'project' }, f.slug))] : null)
  const picker =
    state.picking &&
    h(
      'div',
      { className: 'picker' },
      state.projects === null
        ? h('span', { className: 'muted' }, T.loading)
        : state.projects.map((p) => {
            const on = d.follows.some((f) => f.account === p.account && String(f.project_id) === String(p.id))
            const label = d.accounts.length > 1 ? `${p.slug} · ${(d.accounts.find((a) => a.id === p.account) || {}).label}` : p.slug
            return h('label', {}, h('input', { type: 'checkbox', checked: on, onchange: (e) => send('follow', { account: p.account, project: String(p.id), slug: p.slug, on: e.target.checked }) }), label)
          }),
    )
  const toggle = () => {
    state.picking = !state.picking
    if (state.picking && state.projects === null) send('projects')
    render()
  }
  return [
    h('div', { className: 'bar' }, followed, h('span', { className: 'status' }), h('button', { onclick: toggle }, state.picking ? T.done : T.choose)),
    picker,
    (d.errors || []).map((e) => h('p', { className: 'error' }, e)),
    d.follows.length > 0 && triageView(),
    !d.follows.length
      ? h('p', { className: 'muted empty' }, T.noFollow)
      : !d.issues.length
        ? h('p', { className: 'muted empty' }, T.noIssue)
        : issuesView(),
    d.follows.length > 0 && h('p', { className: 'muted' }, d.auto ? T.auto : T.manual),
  ]
}

function detailView() {
  const x = found()
  if (!x) {
    state.open = null
    return listView()
  }
  const { account, issue: i, proposal: p } = x
  const n = noiseOf(account, i.id)
  const act = (type, extra = {}) => () => {
    state.busy = true
    state.error = state.note = ''
    send(type, { account, id: i.id, ...extra })
    render()
  }
  return [
    h(
      'div',
      { className: 'bar' },
      h('button', { onclick: () => { state.open = null; render() } }, T.back),
      h('span', { className: 'status' }),
      h('button', { className: 'primary', disabled: state.busy, onclick: act('fix') }, T.fix),
      h('button', { disabled: state.busy || p?.status === 'running', onclick: act('propose') }, p ? T.repropose : T.propose),
      h('button', { disabled: state.busy, onclick: act('status', { status: 'resolved' }) }, T.resolve),
      h('button', { disabled: state.busy, onclick: act('status', { status: 'ignored' }) }, T.ignore),
    ),
    h(
      'div',
      { className: 'detail' },
      h('h2', {}, `${i.short_id} · ${i.title}`),
      i.culprit && h('p', { className: 'muted' }, i.culprit),
      h(
        'dl',
        { className: 'stats' },
        [
          [T.stats.count, number.format(Number(i.count) || 0)],
          [T.stats.users, number.format(i.users || 0)],
          [T.stats.first, `${fullDate(i.first_seen)} · ${when(i.first_seen)}`],
          [T.stats.last, `${fullDate(i.last_seen)} · ${when(i.last_seen)}`],
          [T.stats.status, T.substatus[i.substatus] || i.substatus || '—'],
          [T.stats.level, i.level || '—'],
          [T.stats.project, i.project],
        ].map(([k, v]) => h('div', {}, h('dt', {}, k), h('dd', {}, v))),
      ),
    ),
    n && h('p', { className: 'muted' }, T.noiseNote(T.reasons[n.reason] || n.reason, n.why)),
    state.note && h('p', { className: 'muted' }, state.note),
    !p
      ? h('p', { className: 'muted' }, T.noProposal)
      : p.status === 'running'
        ? h('p', { className: 'muted' }, T.proposing)
        : p.status === 'failed'
          ? [h('p', { className: 'error' }, T.failedText), h('pre', {}, p.text)]
          : proposalView(p.text),
    h('details', {}, h('summary', {}, T.stack), h('pre', {}, state.context[i.id] || T.loading)),
  ]
}

function render() {
  const d = state.data
  const head = h(
    'header',
    { className: 'bar' },
    h('strong', {}, 'Sentry'),
    h('span', { className: 'status muted', role: 'status' }, d ? '' : T.loading),
    !state.open && h('button', { disabled: !d, onclick: () => { state.error = ''; state.data = null; send('refresh'); render() } }, T.refresh),
  )
  const body = !d ? [] : state.open ? detailView() : listView()
  document.getElementById('app').replaceChildren(head, ...[state.error && h('p', { className: 'error' }, state.error), body].flat(Infinity).filter(Boolean))
}

squadron.on((msg) => {
  if (!msg || (msg.to !== me && msg.to !== '*')) return
  switch (msg.type) {
    case 'init':
      state.data = msg
      state.busy = state.sorting = false
      state.days ??= msg.days
      state.sort = msg.sort || state.sort
      state.selected = new Set((msg.issues || []).map(keyOf).filter((k) => state.selected.has(k)))
      return render()
    case 'stale':
      state.stale = msg
      state.staleBusy = false
      state.days = msg.days
      return render()
    case 'projects':
      state.projects = msg.projects
      if (msg.errors?.length) state.error = msg.errors.join(' · ')
      return render()
    case 'detail':
      state.context[msg.id] = msg.context
      return render()
    case 'proposal': {
      const x = (state.data?.issues || []).find((y) => y.account === msg.account && y.issue.id === msg.id)
      if (x) x.proposal = msg.proposal
      if (msg.to === me) state.busy = false
      return render()
    }
    case 'opened':
      state.busy = false
      state.note = T.opened
      return render()
    case 'changed':
      return send('refresh')
    case 'error':
      state.busy = state.sorting = state.staleBusy = false
      state.error = msg.message
      if (!state.data) state.data = { accounts: [], follows: [], issues: [], errors: [] }
      return render()
  }
})

render()
send('ready')
