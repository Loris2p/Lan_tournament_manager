document.addEventListener('DOMContentLoaded', () => {
    const btnLogin = document.getElementById('btn-show-login');
    const btnReg = document.getElementById('btn-show-register');
    const wrapLogin = document.getElementById('login-form-wrapper');
    const wrapReg = document.getElementById('register-form-wrapper');

    if (!btnLogin || !btnReg || !wrapLogin || !wrapReg) return;

    function showLogin() {
        wrapLogin.style.display = 'block';
        wrapReg.style.display = 'none';
        btnLogin.className = 'ar-btn ar-btn-solid';
        btnReg.className = 'ar-btn ar-btn-ghost';
    }

    function showRegister() {
        wrapLogin.style.display = 'none';
        wrapReg.style.display = 'block';
        btnLogin.className = 'ar-btn ar-btn-ghost';
        btnReg.className = 'ar-btn ar-btn-solid';
    }

    btnLogin.addEventListener('click', showLogin);
    btnReg.addEventListener('click', showRegister);

    const params = new URLSearchParams(window.location.search);
    const err = params.get('error');
    if (err === 'team_name_taken' || err === 'captain_name_taken' || err === 'passwords_dont_match' || err === 'invalid_register') {
        showRegister();
    }
});
