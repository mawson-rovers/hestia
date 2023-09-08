(function () { // prevent leakage into global scope
    const boards = ["top", "bottom"];
    const host = document.body.dataset.fetchHost || "";
    const coreTemperature = getFieldElements('core-temperature');
    const heaterPower = getFieldElements('heater-power');
    const heaterMode = getFieldElements('heater-mode');
    const heaterDuty = getFieldElements('heater-duty');
    const targetTemp = getFieldElements('target-temp');
    const targetSensor = getFieldElements('target-sensor');
    const targetSensorLabel = getFieldElements('target-sensor-label');
    const boardChartElement = getFieldElements('board-chart');

    function getFieldElements(id) {
        return Object.fromEntries(
            boards.map(board => [board, document.getElementById(`${id}-${board}`)])
        );
    }

    function updateStatus(status_data) {
        boards.forEach(board => {
            let data = status_data[board] || {};
            coreTemperature[board].value = data["target_sensor_temp"] ?? "n/a";
            heaterPower[board].value = data["heater_power"] ?? "n/a";
            heaterMode[board].value = data['heater_mode'] ?? "n/a";
            heaterDuty[board].value = data["heater_duty"] ?? "n/a";
            targetTemp[board].value = data["target_temp"] ?? "n/a";
            targetSensor[board].value = data["target_sensor"] ?? "n/a";
            targetSensorLabel[board].innerText = targetSensor[board].value;
            
            window.boardChart ??= {};
            if (!window.boardChart[board] && data['sensor_info']) {
                window.boardChart[board] = newBoardChart(boardChartElement[board], data['sensor_info']);
            }
        });
    }

    function newBoardChart(ctx, data) {
        let mounted = 0;
        return new Chart(ctx, {
            type: 'scatter',
            data: {
                datasets: Object.keys(data).map(id => {
                    let x = Math.abs(data[id]['pos_x']);
                    let y = Math.abs(data[id]['pos_y']);
                    if (x === 0.0 && y === 0.0) {
                        mounted += 1;
                        x = 96;
                        y = 92 - mounted * 5;
                    }
                    let colors = window.colorsForSensor(id);
                    return {
                        label: id,
                        data: [{x: x, y: y, temp: null}],
                        unit: data[id]['unit'],
                        borderWidth: 1,
                        borderColor: colors.borderColor,
                        backgroundColor: colors.backgroundColor,
                    };
                })
            },
            options: {
                aspectRatio: 1.2,
                scales: {
                    // scales are adjusted so plot aligns with background image
                    x: {
                        min: 3.0,
                        max: 113.0, // x > 92 used for mounted sensors
                        display: false,
                    },
                    y: {
                        min: 3.0,
                        max: 92.0,
                        display: false,
                    }
                },
                elements: {
                    point: {
                        radius: 8.0,
                        hoverRadius: 12.0,
                    },
                },
                plugins: {
                    legend: {
                        display: false,
                    },
                    tooltip: {
                        callbacks: {
                            label: function (ctx) {
                                let label = ctx.dataset.label || '';
                                if (label) label += ': ';
                                label += ctx.raw.temp !== null ?
                                    `${ctx.raw.temp} ${ctx.dataset.unit}` : "n/a";
                                return label;
                            },
                        },
                    },
                },
            },
        });
    }

    function fetchAndUpdateStatus() {
        fetch(host + '/api/status')
            .then(response => response.json())
            .then(data => updateStatus(data));
    }

    fetchAndUpdateStatus();
    setInterval(fetchAndUpdateStatus, 5000);

    function postStatusUpdate(data) {
        fetch(host + '/api/status', {
            method: 'POST',
            body: JSON.stringify({
                ...data
            }),
            redirect: "follow",
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            },
        })
            .then(response => response.json())
            .then(data => updateStatus(data))
    }

    // heater mode buttons
    document.querySelectorAll(".set-heater-mode").forEach(el => {
        el.addEventListener('click', () => {
            let mode = el.getAttribute("data-mode");
            let board = el.getAttribute("data-board");
            postStatusUpdate({ 'heater_mode': mode, 'board': board });
        });
    });

    // heater duty buttons
    document.querySelectorAll(".set-heater-duty").forEach(el => {
        el.addEventListener('click', () => {
            let duty = Number(el.getAttribute('data-duty'));
            let board = el.getAttribute("data-board");
            postStatusUpdate({ 'heater_duty': duty, 'board': board });
        });
    });

    document.querySelectorAll(".set-custom-heater-duty").forEach(el => {
        const customDuty = document.getElementById("custom-heater-duty");
        el.addEventListener('click', () => {
            let duty = Math.floor(Number(customDuty.value) * 255);
            let board = el.getAttribute("data-board");
            postStatusUpdate({ 'heater_duty': duty, 'board': board });
        });
    });

    // target temperature buttons
    document.querySelectorAll(".set-target-temp").forEach(el => {
        el.addEventListener('click', () => {
            let temp = Number(el.getAttribute('data-temp'));
            let board = el.getAttribute("data-board");
            postStatusUpdate({ 'target_temp': temp, 'board': board });
        });
    });

    // target sensor buttons
    document.querySelectorAll(".set-target-sensor").forEach(el => {
        el.addEventListener('click', () => {
            let sensor = el.getAttribute('data-sensor');
            let board = el.getAttribute("data-board");
            postStatusUpdate({ 'target_sensor': sensor, 'board': board });
        });
    });

    // flip board link
    document.getElementById("board-flip")?.addEventListener('click', (ev) => {
        document.getElementById("board-chart-container").classList.toggle("flip");
        ev.preventDefault();
        ev.stopPropagation();
    });

    if (host) {
        document.querySelectorAll(".api-links a").forEach(link => {
            link.setAttribute("href", host + link.getAttribute("href"));
        });
    }
})();
