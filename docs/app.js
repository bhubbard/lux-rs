document.addEventListener('DOMContentLoaded', () => {
  const urlInput = document.getElementById('video-url');
  const streamSelect = document.getElementById('stream-quality');
  const outputDir = document.getElementById('output-dir');
  const threadCount = document.getElementById('thread-count');
  const infoOnly = document.getElementById('info-only');
  const noCaptions = document.getElementById('no-captions');
  const playlist = document.getElementById('playlist');
  const commandEl = document.getElementById('generated-command');
  const copyBtn = document.getElementById('copy-btn');

  function updateCommand() {
    const parts = ['lux'];

    if (infoOnly.checked) {
      parts.push('-i');
    }

    const url = (urlInput.value || '').trim();
    if (url) {
      parts.push(`"${url}"`);
    }

    const stream = streamSelect.value;
    if (stream && !infoOnly.checked) {
      parts.push(`-s ${stream}`);
    }

    const out = (outputDir.value || '').trim();
    if (out && out !== '.' && !infoOnly.checked) {
      parts.push(`-o ${out}`);
    }

    const threads = parseInt(threadCount.value, 10);
    if (threads && threads !== 4 && !infoOnly.checked) {
      parts.push(`--threads ${threads}`);
    }

    if (noCaptions.checked && !infoOnly.checked) {
      parts.push('-C');
    }

    if (playlist.checked) {
      parts.push('-p');
    }

    commandEl.textContent = parts.join(' ');
  }

  [urlInput, streamSelect, outputDir, threadCount, infoOnly, noCaptions, playlist].forEach(el => {
    el.addEventListener('input', updateCommand);
    el.addEventListener('change', updateCommand);
  });

  copyBtn.addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(commandEl.textContent);
      copyBtn.textContent = 'Copied!';
      setTimeout(() => {
        copyBtn.textContent = 'Copy';
      }, 2000);
    } catch (e) {
      console.error(e);
    }
  });

  updateCommand();
});
