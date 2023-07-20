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

    function updateLogFiles(data) {
        if (!arraysEqual(data, lastLogs)) {
            logFileList.replaceChildren(...data.map(logFile => {
                return createElementFromHtml(`<li><a href="${logFile.url}">${logFile.name}</a></li>`);
            }));
        }
        lastLogs = data;
    }

    fetchAndUpdateLogFiles();
    setInterval(fetchAndUpdateLogFiles, 60 * 1000);
}());