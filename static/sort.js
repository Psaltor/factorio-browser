// Client-side sorting for server cards
(function() {
    const grid = document.querySelector('.server-grid');
    const buttons = document.querySelectorAll('.sort-button');
    
    if (!grid || !buttons.length) return;
    
    buttons.forEach(btn => {
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
            
            // Update button states
            buttons.forEach(b => {
                b.classList.remove('active');
                b.dataset.dir = '';
                b.querySelector('.sort-arrow').textContent = '';
            });
            
            btn.classList.add('active');
            btn.dataset.dir = dir;
            btn.querySelector('.sort-arrow').textContent = dir === 'desc' ? '▼' : '▲';
            
            // Sort the cards
            sortCards(sortBy, dir);
        });
    });
    
    function sortCards(sortBy, dir) {
        const cards = Array.from(grid.querySelectorAll('.server-card'));
        
        cards.sort((a, b) => {
            const aVal = parseInt(a.dataset[sortBy]) || 0;
            const bVal = parseInt(b.dataset[sortBy]) || 0;
            return dir === 'desc' ? bVal - aVal : aVal - bVal;
        });
        
        // Re-append in sorted order
        cards.forEach(card => grid.appendChild(card));
    }
    
    // Initial sort by players descending
    sortCards('players', 'desc');
})();

