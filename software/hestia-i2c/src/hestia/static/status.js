(function () { // prevent leakage into global scope
    const coreTemperature = document.getElementById('core-temperature');
    const heaterMode = document.getElementById('heater-mode');
    const heaterToggle = document.getElementById('heater-toggle');

    function updateStatus(data) {
        coreTemperature.value = "center_temp" in data ? data["center_temp"] : "n/a";
        if ('heater_enabled' in data) {
            heaterMode.value = data['heater_enabled'] ? "ON" : "OFF";
        } else {
            heaterMode.value = "n/a";
        }
    }

    fetch('/api/status')
        .then(response => response.json())
        .then(data => updateStatus(data));

    heaterToggle.addEventListener('click', (e) => {
        let newValue = heaterMode.value !== "ON";
        let data = JSON.stringify({
            'heater_enabled': newValue,
        });

        fetch('/api/status', {
            method: 'POST',
            body: data,
            redirect: "follow",
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            },
        })
            .then(response => response.json())
            .then(data => updateStatus(data))
    });
})();
