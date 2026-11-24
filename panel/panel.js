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
      noFollow: 'Aucun projet Sentry suivi pour ce projet Squadron. Choisissez-les : les erreurs arrivées ensuite seront notifiées, avec une correction proposée.',
      noIssue: 'Aucune erreur non résolue ces 14 derniers jours.',
      events: (n) => `${n} événement${n > 1 ? 's' : ''}`,
      users: (n) => `${n} utilisateur${n > 1 ? 's' : ''}`,
      seen: 'vue',
      regressed: 'revenue',
      ready: 'correction proposée',
      running: 'proposition en cours…',
      failed: 'proposition en échec',
      back: '← Erreurs',
      fix: 'Corriger avec un agent',
      propose: 'Proposer une correction',
      repropose: 'Reproposer',
      resolve: 'Résoudre',
      ignore: 'Ignorer',
      stack: 'Stack trace et contexte envoyés à Claude',
      noProposal: 'Pas encore de proposition. Claude Code lira le dépôt sans le modifier et proposera un diagnostic et un correctif.',
      proposing: 'Claude Code lit le code et prépare une proposition (jusqu’à 2 minutes)…',
      failedText: 'La proposition n’a pas abouti :',
      opened: 'La fenêtre Nouvel agent est ouverte, avec l’erreur en contexte.',
      auto: 'Chaque erreur nouvelle ou revenue reçoit une proposition automatiquement.',
      manual: 'Propositions automatiques coupées dans les réglages.',
    }
  : {
      refresh: 'Refresh',
      loading: 'Loading…',
      noAccount: 'Add your Sentry organization in Settings › Extensions › Sentry: server, organization slug and personal token.',
      choose: 'Choose Sentry projects',
      done: 'Done',
      following: 'Followed for this project:',
      noFollow: 'No Sentry project is followed for this Squadron project. Choose them: errors that come in afterwards will be notified, with a proposed fix.',
      noIssue: 'No unresolved error in the last 14 days.',
      events: (n) => `${n} event${n > 1 ? 's' : ''}`,
      users: (n) => `${n} user${n > 1 ? 's' : ''}`,
      seen: 'seen',
      regressed: 'back',
      ready: 'fix proposed',
      running: 'proposing…',
      failed: 'proposal failed',
      back: '← Errors',
      fix: 'Fix with an agent',
      propose: 'Propose a fix',
      repropose: 'Propose again',
      resolve: 'Resolve',
      ignore: 'Ignore',
      stack: 'Stack trace and context sent to Claude',
      noProposal: 'No proposal yet. Claude Code will read the repository without changing it and propose a diagnosis and a fix.',
      proposing: 'Claude Code is reading the code and preparing a proposal (up to 2 minutes)…',
      failedText: 'The proposal did not finish:',
      opened: 'The New agent window is open, with the error as context.',
      auto: 'Each new or returning error gets a proposal automatically.',
      manual: 'Automatic proposals are off in the settings.',
    }

const state = { data: null, projects: null, picking: false, open: null, context: {}, error: '', note: '', busy: false }
const send = (type, extra = {}) => squadron.send({ ...extra, type, to: me, lang })

const relative = new Intl.RelativeTimeFormat(lang, { numeric: 'auto' })
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

function openIssue(account, id) {
  state.open = { account, id }
  state.note = ''
  if (!state.context[id]) send('detail', { account, id })
  render()
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
    !d.follows.length
      ? h('p', { className: 'muted empty' }, T.noFollow)
      : !d.issues.length
        ? h('p', { className: 'muted empty' }, T.noIssue)
        : h(
            'ul',
            { className: 'issues' },
            d.issues.map(({ account, issue: i, proposal }) =>
              h(
                'li',
                {},
                h(
                  'button',
                  { className: 'issue', onclick: () => openIssue(account, i.id) },
                  h('span', { className: 'key' }, i.short_id),
                  h('span', { className: 'title' }, i.title),
                  h('span', {}, i.substatus === 'regressed' && h('span', { className: 'tag err' }, T.regressed), ' ', tag(proposal)),
                  h('span', { className: 'meta muted' }, [i.project, T.events(Number(i.count) || 0), T.users(i.users), `${T.seen} ${when(i.last_seen)}`].join(' · ')),
                ),
              ),
            ),
          ),
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
      h('p', { className: 'muted' }, [i.culprit, T.events(Number(i.count) || 0), T.users(i.users), `${T.seen} ${when(i.last_seen)}`].filter(Boolean).join(' · ')),
    ),
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
      state.busy = false
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
      state.busy = false
      state.error = msg.message
      if (!state.data) state.data = { accounts: [], follows: [], issues: [], errors: [] }
      return render()
  }
})

render()
send('ready')
