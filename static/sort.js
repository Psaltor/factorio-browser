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
            const aVal = parseInt(a.dataset[sortBy]) || 0;
            const bVal = parseInt(b.dataset[sortBy]) || 0;
            return dir === 'desc' ? bVal - aVal : aVal - bVal;
        });
        
        // Re-append in sorted order
        items.forEach(item => grid.appendChild(item));
    }
    
    // Initialize
    loadPreferences();
})();
