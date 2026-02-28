document.addEventListener('DOMContentLoaded', () => {
    const globalStatus = document.getElementById('global-status');
    const btnPause = document.getElementById('btn-pause');
    const btnResume = document.getElementById('btn-resume');
    const threadList = document.getElementById('thread-list');
    const loungeMessages = document.getElementById('lounge-messages');
    const bossMsgInput = document.getElementById('boss-msg');
    const btnSend = document.getElementById('btn-send');

    // Fetch and render threads
    async function updateThreads() {
        try {
            const response = await fetch('/threads');
            const threads = await response.json();

            if (threads.length === 0) return;

            threadList.innerHTML = '';
            threads.forEach(thread => {
                const card = document.createElement('div');
                card.className = `thread-card ${thread.status.toLowerCase()}`;
                card.innerHTML = `
                    <div class="thread-header">
                        <span class="thread-id">#${thread.id}</span>
                        <span class="thread-status-dot"></span>
                    </div>
                    <h3 class="thread-title">${thread.title}</h3>
                    <div class="thread-progress-container">
                        <div class="thread-progress-bar" style="width: ${thread.progress}%"></div>
                    </div>
                    <div class="thread-footer">
                        <span>PROGRESS: ${thread.progress}%</span>
                        <span>STATE: ${thread.status.toUpperCase()}</span>
                    </div>
                `;
                threadList.appendChild(card);
            });

            // Update global progress
            const totalProgress = threads.reduce((acc, t) => acc + t.progress, 0) / threads.length;
            document.getElementById('factory-progress').style.width = `${totalProgress}%`;
            document.getElementById('progress-text').textContent = `${Math.round(totalProgress)}% (${threads.length} Threads)`;

        } catch (error) {
            console.error('Failed to fetch threads:', error);
        }
    }

    btnPause.addEventListener('click', async () => {
        await fetch('/pause', { method: 'POST' });
        updateStatus();
    });

    btnResume.addEventListener('click', async () => {
        await fetch('/resume', { method: 'POST' });
        updateStatus();
    });

    btnSend.addEventListener('click', () => {
        const msg = bossMsgInput.value;
        if (msg) {
            appendMessage('BOSS', '👑', 'Dogsinatas', msg, 'boss');
            bossMsgInput.value = '';
        }
    });

    function appendMessage(role, icon, name, content, type) {
        const msgEl = document.createElement('div');
        msgEl.className = `message ${type}`;
        msgEl.innerHTML = `
            <span class="msg-meta">[${role}] ${icon} ${name}:</span>
            <span class="msg-content">${content}</span>
        `;
        loungeMessages.appendChild(msgEl);
        loungeMessages.scrollTop = loungeMessages.scrollHeight;
    }

    // Initial check
    updateStatus();
    updateThreads();
    setInterval(() => {
        updateStatus();
        updateThreads();
    }, 2000);
});
