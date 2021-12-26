export function doneLoading() {
    document.getElementById('button-loading').remove();
    document.getElementById('button').removeAttribute("hidden");
    document.getElementById('footer').removeAttribute("hidden");
}
