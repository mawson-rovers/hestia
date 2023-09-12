(function () { // prevent leakage into global scope
    const host = document.body.dataset.fetchHost || "";
    const logFileList = document.getElementById('log-files');

    let lastLogs = [];

    function arraysEqual(a, b) {
        return a.length === b.length &&
            a.every((value, index) => value === b[index]);
    }

    /**
     * @param {String} HTML representing a single element
     * @return {Element}
     */
    function createElementFromHtml(html) {
        var template = document.createElement('template');
        html = html.trim(); // Never return a text node of whitespace as the result
        template.innerHTML = html;
        return template.content.firstChild;
    }

    function fetchAndUpdateLogFiles() {
        fetch(host + '/api/log_files')
            .then(response => response.json())
            .then(data => updateLogFiles(data));
    }

    const createLogFileElement = (logFile) =>
        createElementFromHtml(`<li><a href="${logFile.url}">${logFile.name}</a></li>`);
    const detailsTemplate = `<details class="text-sm my-2">
        <summary>Older files</summary>
        <ul class="list-disc pl-4 text-sm"></ul>
        </details>`;

    function updateLogFiles(data) {
        if (!arraysEqual(data, lastLogs)) {
            logFileList.replaceChildren(...data.filter((_, i) => i <= 10)
                .map(createLogFileElement));
            if (data.length > 10) {
                let details = logFileList.parentElement.appendChild(createElementFromHtml(detailsTemplate));
                details.lastElementChild.append(...data.filter((_, i) => i > 10).map(createLogFileElement));
            }
        }
        lastLogs = data;
    }

    fetchAndUpdateLogFiles();
    setInterval(fetchAndUpdateLogFiles, 60 * 1000);
}());