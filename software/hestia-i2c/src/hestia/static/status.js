(function () { // prevent leakage into global scope
    const coreTemperature = document.getElementById('core-temperature');
    const heaterMode = document.getElementById('heater-mode');
    const heaterPowerLevel = document.getElementById('heater-power-level');
    const heaterToggle = document.getElementById('heater-toggle');

    function updateStatus(data) {
        coreTemperature.value = "center_temp" in data ? data["center_temp"] : "n/a";
        if ('heater_enabled' in data) {
            heaterMode.value = data['heater_enabled'] ? "ON" : "OFF";
        } else {
            heaterMode.value = "n/a";
        }
        heaterPowerLevel.value = ("heater_pwm_freq" in data && data["heater_pwm_freq"]) ?
            data["heater_pwm_freq"] : "n/a"
    }

    function fetchAndUpdateStatus() {
        fetch('/api/status')
            .then(response => response.json())
            .then(data => updateStatus(data));
    }

    fetchAndUpdateStatus();
    setInterval(fetchAndUpdateStatus, 5000);

    function postStatusUpdate(data) {
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
    }

    heaterToggle.addEventListener('click', (e) => {
        let newValue = heaterMode.value !== "ON";
        postStatusUpdate(JSON.stringify({
            'heater_enabled': newValue,
        }));
    });

    document.querySelectorAll(".set-heater-power").forEach(el => {
        el.addEventListener('click', () => {
            let powerLevel = Number(el.getAttribute('data-power-level'));
            postStatusUpdate(JSON.stringify({
                'heater_pwm_freq': powerLevel,
            }));
        });
    });
})();
