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
const distanceFunctions = {
  levenshtein: { fn: distance.levenshtein, type: 'distance' },
  normalizedLevenshtein: { fn: distance.normalizedLevenshtein, type: 'similarity' },
  damerauLevenshtein: { fn: distance.damerauLevenshtein, type: 'distance' },
  jaro: { fn: distance.jaro, type: 'similarity' },
  jaroWinkler: { fn: distance.jaroWinkler, type: 'similarity' },
  sorensenDice: { fn: distance.sorensenDice, type: 'similarity' },
  tokenSortRatio: { fn: distance.tokenSortRatio, type: 'similarity' },
  tokenSetRatio: { fn: distance.tokenSetRatio, type: 'similarity' },
  partialRatio: { fn: distance.partialRatio, type: 'similarity' },
  weightedRatio: { fn: distance.weightedRatio, type: 'similarity' },
};

// --- DOM refs ---
const searchInput = document.getElementById('search-input');
const searchResults = document.getElementById('search-results');
const searchTiming = document.getElementById('search-timing');
const optPositions = document.getElementById('opt-positions');
const distA = document.getElementById('dist-a');
const distB = document.getElementById('dist-b');
const algorithmSelect = document.getElementById('algorithm');
const distanceResult = document.getElementById('distance-result');
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
  const algo = algorithmSelect.value;
  const config = distanceFunctions[algo] ?? distanceFunctions.jaroWinkler;

  if (!a && !b) {
    distanceResult.innerHTML = '';
    return;
  }

  const result = config.fn(a, b);
  const label = config.type === 'distance' ? 'Distance' : 'Similarity';

  let display;
  if (config.type === 'distance') {
    display = result;
  } else {
    display = result.toFixed(4);
  }

  distanceResult.innerHTML = `
    <span class="distance-label">${label}</span>
    ${display}
  `;
}

// --- Utils ---
function escapeHtml(str) {
  return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

// --- Init ---
function init() {
  // Verify WASM is loaded by calling a simple function
  try {
    distance.levenshtein('a', 'b');
  } catch {
    loadingOverlay.querySelector('p').textContent =
      'Failed to load WASM module. Ensure CORS headers are set.';
    return;
  }

  loadingOverlay.classList.add('hidden');

  // Event listeners
  searchInput.addEventListener('input', runSearch);
  optPositions.addEventListener('change', runSearch);
  distA.addEventListener('input', runDistance);
  distB.addEventListener('input', runDistance);
  algorithmSelect.addEventListener('change', runDistance);

  // Initial state
  runDistance();
  searchInput.focus();
}

init();
