import * as distance from 'rapid-fuzzy';
import { highlight, search } from 'rapid-fuzzy';

// --- Sample data ---
const sampleItems = [
  'TypeScript',
  'JavaScript',
  'Python',
  'Rust',
  'Go',
  'Ruby',
  'Swift',
  'Kotlin',
  'Scala',
  'Haskell',
  'Clojure',
  'Elixir',
  'Erlang',
  'Lua',
  'Perl',
  'PHP',
  'Java',
  'C#',
  'C++',
  'C',
  'Zig',
  'Nim',
  'Dart',
  'Julia',
  'R',
  'MATLAB',
  'Fortran',
  'COBOL',
  'Assembly',
  'Objective-C',
  'Visual Basic',
  'F#',
  'OCaml',
  'Racket',
  'Scheme',
  'Common Lisp',
  'Crystal',
  'V',
  'Odin',
  'Mojo',
  'Gleam',
  'Roc',
  'Unison',
  'Bend',
];

// --- Distance algorithms ---
const algorithms = [
  { label: 'Levenshtein', fn: distance.levenshtein, type: 'distance' },
  { label: 'Damerau-Levenshtein', fn: distance.damerauLevenshtein, type: 'distance' },
  { label: 'Normalized Levenshtein', fn: distance.normalizedLevenshtein, type: 'similarity' },
  { label: 'Jaro', fn: distance.jaro, type: 'similarity' },
  { label: 'Jaro-Winkler', fn: distance.jaroWinkler, type: 'similarity' },
  { label: 'Sørensen-Dice', fn: distance.sorensenDice, type: 'similarity' },
  { label: 'Token Sort Ratio', fn: distance.tokenSortRatio, type: 'similarity' },
  { label: 'Token Set Ratio', fn: distance.tokenSetRatio, type: 'similarity' },
  { label: 'Partial Ratio', fn: distance.partialRatio, type: 'similarity' },
  { label: 'Weighted Ratio', fn: distance.weightedRatio, type: 'similarity' },
];

// --- DOM refs ---
const searchInput = document.getElementById('search-input');
const searchResults = document.getElementById('search-results');
const searchTiming = document.getElementById('search-timing');
const optPositions = document.getElementById('opt-positions');
const distA = document.getElementById('dist-a');
const distB = document.getElementById('dist-b');
const algorithmGrid = document.getElementById('algorithm-grid');
const loadingOverlay = document.getElementById('loading-overlay');

// --- Search ---
function runSearch() {
  const query = searchInput.value;

  if (!query.trim()) {
    searchResults.innerHTML = '<p class="placeholder">Results will appear here as you type.</p>';
    searchTiming.textContent = '';
    return;
  }

  const usePositions = optPositions.checked;

  const start = performance.now();
  const results = search(query, sampleItems, {
    includePositions: usePositions,
    maxResults: 20,
  });
  const elapsed = performance.now() - start;

  const noun = results.length === 1 ? 'result' : 'results';
  searchTiming.textContent = `${results.length} ${noun} in ${elapsed.toFixed(2)} ms`;

  if (results.length === 0) {
    searchResults.innerHTML = '<p class="placeholder">No matches found.</p>';
    return;
  }

  searchResults.innerHTML = results
    .map((r) => {
      const text =
        usePositions && r.positions
          ? highlight(r.item, r.positions, '<mark>', '</mark>')
          : escapeHtml(r.item);

      const scoreClass = r.score >= 0.8 ? 'score-high' : r.score >= 0.5 ? 'score-mid' : 'score-low';

      const matchType = r.matchType
        ? `<span class="match-type match-type-${r.matchType.toLowerCase()}">${r.matchType}</span>`
        : '';

      return `<div class="result-item">
        <span class="text">${text}</span>
        ${matchType}
        <span class="score ${scoreClass}">${r.score.toFixed(3)}</span>
      </div>`;
    })
    .join('');
}

// --- Distance ---
function runDistance() {
  const a = distA.value;
  const b = distB.value;

  if (!a && !b) {
    algorithmGrid.innerHTML = '';
    return;
  }

  const distanceAlgos = algorithms.filter((algo) => algo.type === 'distance');
  const similarityAlgos = algorithms.filter((algo) => algo.type === 'similarity');

  let html = '';

  html += '<div class="algo-group"><div class="algo-group-label">Distance</div>';
  for (const algo of distanceAlgos) {
    const value = algo.fn(a, b);
    html += `<div class="algo-item">
      <span class="algo-name">${algo.label}</span>
      <span class="algo-value">${value}</span>
    </div>`;
  }
  html += '</div>';

  html += '<div class="algo-group"><div class="algo-group-label">Similarity</div>';
  for (const algo of similarityAlgos) {
    const value = algo.fn(a, b);
    const pct = (value * 100).toFixed(1);
    const colorClass = value >= 0.8 ? 'bar-high' : value >= 0.5 ? 'bar-mid' : 'bar-low';
    html += `<div class="algo-item">
      <span class="algo-name">${algo.label}</span>
      <span class="algo-value">${value.toFixed(4)}</span>
      <div class="algo-bar"><div class="algo-bar-fill ${colorClass}" style="width:${pct}%"></div></div>
    </div>`;
  }
  html += '</div>';

  algorithmGrid.innerHTML = html;
}

// --- Utils ---
function escapeHtml(str) {
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

// --- Init ---
function init() {
  try {
    distance.levenshtein('a', 'b');
  } catch {
    loadingOverlay.querySelector('p').textContent =
      'Failed to load WASM module. Please try a modern browser (Chrome, Firefox, Edge).';
    return;
  }

  loadingOverlay.classList.add('hidden');

  searchInput.addEventListener('input', runSearch);
  optPositions.addEventListener('change', runSearch);
  distA.addEventListener('input', runDistance);
  distB.addEventListener('input', runDistance);

  // Show results immediately
  searchInput.value = 'typscript';
  runSearch();
  runDistance();
  searchInput.focus();
  searchInput.select();
}

init();
