// Handle view transitions on back/forward navigation
(function() {
    // Skip if view transitions are not supported
    if (!('ViewTransition' in window)) return;
    
    // Listen for pagereveal to handle back/forward navigation
    window.addEventListener('pagereveal', (event) => {
        // If there's no view transition, nothing to do
        if (!event.viewTransition) return;
        
        // Check if this is a back/forward navigation (traverse)
        const navigationType = performance.getEntriesByType('navigation')[0]?.type;
        const isTraversal = navigationType === 'back_forward';
        
        // Also check using Navigation API if available
        const navEntry = window.navigation?.currentEntry;
        const isBackNavigation = navEntry && window.navigation?.transition?.navigationType === 'traverse';
        
        if (isTraversal || isBackNavigation) {
            // Add a class to indicate back navigation for CSS styling
            document.documentElement.classList.add('back-navigation');
            
            // Remove the class after the transition completes
            event.viewTransition.finished.then(() => {
                document.documentElement.classList.remove('back-navigation');
            }).catch(() => {
                document.documentElement.classList.remove('back-navigation');
            });
        }
    });
    
    // Ensure video continues playing after BFCache restore
    window.addEventListener('pageshow', (event) => {
        if (event.persisted) {
            // Page was restored from BFCache
            const video = document.querySelector('.video-background');
            if (video && video.paused) {
                video.play().catch(() => {});
            }
        }
    });
})();

// Tag filtering toggle function
function toggleTag(tag) {
    const input = document.getElementById('tags-input');
    if (!input) return;
    
    const currentValue = input.value;
    const tags = currentValue ? currentValue.split(',').filter(t => t.trim()) : [];
    
    const index = tags.indexOf(tag);
    if (index === -1) {
        // Add tag
        tags.push(tag);
    } else {
        // Remove tag
        tags.splice(index, 1);
    }
    
    input.value = tags.join(',');
}

// Client-side sorting and view toggle for server list
(function() {
    const grid = document.querySelector('.server-grid');
    const sortButtons = document.querySelectorAll('.sort-button');
    const viewButtons = document.querySelectorAll('.view-btn');
    
    if (!grid) return;
    
    const STORAGE_KEY_VIEW = 'factorio-browser-view';
    const STORAGE_KEY_SORT = 'factorio-browser-sort';
    
    // Load saved preferences
    function loadPreferences() {
        try {
            // Load view preference
            const savedView = localStorage.getItem(STORAGE_KEY_VIEW);
            if (savedView === 'list') {
                setView('list');
            }
            
            // Load sort preference
            const savedSort = localStorage.getItem(STORAGE_KEY_SORT);
            if (savedSort) {
                const [sortBy, dir] = savedSort.split(':');
                if (sortBy && dir) {
                    applySort(sortBy, dir);
                    return;
                }
            }
        } catch (e) {
            // localStorage not available
        }
        
        // Default sort
        applySort('players', 'desc');
    }
    
    // Save preferences
    function saveViewPref(view) {
        try {
            localStorage.setItem(STORAGE_KEY_VIEW, view);
        } catch (e) {}
    }
    
    function saveSortPref(sortBy, dir) {
        try {
            localStorage.setItem(STORAGE_KEY_SORT, `${sortBy}:${dir}`);
        } catch (e) {}
    }
    
    // View toggle
    function setView(view) {
        if (view === 'list') {
            grid.classList.add('list-view');
        } else {
            grid.classList.remove('list-view');
        }
        
        viewButtons.forEach(btn => {
            btn.classList.toggle('active', btn.dataset.view === view);
        });
        
        saveViewPref(view);
    }
    
    viewButtons.forEach(btn => {
        btn.addEventListener('click', () => {
            setView(btn.dataset.view);
        });
    });
    
    // Sorting
    function applySort(sortBy, dir) {
        // Update button states
        sortButtons.forEach(btn => {
            const isActive = btn.dataset.sort === sortBy;
            btn.classList.toggle('active', isActive);
            btn.dataset.dir = isActive ? dir : '';
            btn.querySelector('.sort-arrow').textContent = isActive ? (dir === 'desc' ? '▼' : '▲') : '';
        });
        
        // Sort the items
        sortItems(sortBy, dir);
        saveSortPref(sortBy, dir);
    }
    
    sortButtons.forEach(btn => {
        btn.addEventListener('click', () => {
            const sortBy = btn.dataset.sort;
            const wasActive = btn.classList.contains('active');
            
            // Toggle direction if already active, otherwise default to desc
            let dir = btn.dataset.dir || 'desc';
            if (wasActive) {
                dir = dir === 'desc' ? 'asc' : 'desc';
            } else {
                dir = 'desc';
            }
            
            applySort(sortBy, dir);
        });
    });
    
    function sortItems(sortBy, dir) {
        const items = Array.from(grid.querySelectorAll('.server-item'));
        
        items.sort((a, b) => {
            if (sortBy === 'name') {
                // String comparison for name sorting
                // desc (▼) = A→Z, asc (▲) = Z→A
                const aVal = a.dataset[sortBy] || '';
                const bVal = b.dataset[sortBy] || '';
                const cmp = aVal.localeCompare(bVal);
                return dir === 'desc' ? cmp : -cmp;
            } else {
                // Numeric comparison for players, time, etc.
                const aVal = parseInt(a.dataset[sortBy]) || 0;
                const bVal = parseInt(b.dataset[sortBy]) || 0;
                return dir === 'desc' ? bVal - aVal : aVal - bVal;
            }
        });
        
        // Re-append in sorted order
        items.forEach(item => grid.appendChild(item));
    }
    
    // Initialize
    loadPreferences();
})();
