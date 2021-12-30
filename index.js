import init from "./ascii_bomb_ecs_lib.js";

let wasm = undefined;
init("./ascii_bomb_ecs_lib_bg.wasm").then(function (_wasm) {
    wasm = _wasm;
    // TODO: add add indication in case of failure
    wasm.run();
});

// pray that this works :-)
const isTouchDevice = 'ontouchstart' in document.documentElement;

let canvasContainerWidth = undefined;
let canvasContainerHeight = undefined;
let canvas = document.getElementById('bevy-canvas');
function updateCanvasSize() {
    if (canvasContainerHeight != undefined && canvasContainerWidth != undefined) {
        if (canvasContainerHeight / canvasContainerWidth > canvas.offsetHeight / canvas.offsetWidth) {
            canvas.setAttribute('style', 'width:' + canvasContainerWidth + 'px');
        } else {
            canvas.setAttribute('style', 'height:' + canvasContainerHeight + 'px');
        }
    }
}

// override winit resize requests
new ResizeObserver(() => {
    updateCanvasSize();
}).observe(canvas);

let isPortrait = undefined;
let gameContainer = document.getElementById('game-container');
let canvasContainer = document.getElementById('canvas-container');
let canvasInnerContainer = document.getElementById('canvas-inner-container');
function updateCanvasContainerSize() {
    let controls = document.getElementById('controls');
    if (controls != undefined) {
        const rem = parseInt(getComputedStyle(document.documentElement).fontSize);

        const gameContainerWidth = gameContainer.offsetWidth;
        const gameContainerHeight = gameContainer.offsetHeight;

        const controlsPortraitHeightLandscapeWidth = 35 * rem;

        const canvasContainerLandscapeWidth = gameContainer.offsetWidth - 37 * rem;
        const canvasContainerPortraitHeight = gameContainer.offsetHeight - 37 * rem;

        const portraitCanvasContainerIsMoreSquare = Math.abs(canvasContainerPortraitHeight - gameContainerWidth) <= Math.abs(gameContainerHeight - canvasContainerLandscapeWidth);

        // should the orientation change?
        if ((isPortrait || isPortrait == undefined) && !portraitCanvasContainerIsMoreSquare) {
            isPortrait = false;

            canvasInnerContainer.style.display = 'flex';
            canvasInnerContainer.style.justifyContent = 'left';

            controls.style.float = 'right';
            controls.style.padding = '0 1rem';
        } else if ((!isPortrait || isPortrait == undefined) && portraitCanvasContainerIsMoreSquare) {
            isPortrait = true;

            canvasInnerContainer.style.display = 'block';
            canvasInnerContainer.style.justifyContent = 'center';

            controls.style.float = 'none';
            controls.style.padding = '1rem 0';
        }

        if (isPortrait) {
            canvasContainer.style.width = gameContainerWidth + 'px';
            canvasContainer.style.height = canvasContainerPortraitHeight + 'px';

            controls.style.width = gameContainerWidth + 'px';
            controls.style.height = controlsPortraitHeightLandscapeWidth + 'px';
        } else {
            canvasContainer.style.width = canvasContainerLandscapeWidth + 'px';
            canvasContainer.style.height = gameContainerHeight + 'px';

            controls.style.width = controlsPortraitHeightLandscapeWidth + 'px';
            controls.style.height = gameContainerHeight + 'px';
        }
    }

    canvasContainerHeight = canvasContainer.offsetHeight;
    canvasContainerWidth = canvasContainer.offsetWidth;

    updateCanvasSize();
}

window.onresize = updateCanvasContainerSize;

function startGame() {
    document.getElementById('button-box').remove();
    document.getElementById('game-container').removeAttribute("hidden");

    if (isTouchDevice) {
        // go fullscreen
        let elem = document.documentElement;
        if (elem.requestFullscreen) {
            elem.requestFullscreen();
        } else if (elem.webkitRequestFullscreen) { /* Safari */
            elem.webkitRequestFullscreen();
        } else if (elem.msRequestFullscreen) { /* IE11 */
            elem.msRequestFullscreen();
        }
    } else {
        // remove on-screen controls
        document.getElementById('controls').remove();

        canvasContainer.setAttribute('style', 'height:100%');
        canvasContainer.style.heigth = '100%';
    }

    updateCanvasContainerSize();

    canvas.focus();
    wasm.start_game();
}
window.startGame = startGame

function setInputActive(input) {
    wasm.set_input_active(input);
}
window.setInputActive = setInputActive

function toggleFullscreen() {
    if (!document.fullscreenElement &&    // alternative standard method
        !document.mozFullScreenElement && !document.webkitFullscreenElement) {  // current working methods
        if (document.documentElement.requestFullscreen) {
            document.documentElement.requestFullscreen();
        } else if (document.documentElement.mozRequestFullScreen) {
            document.documentElement.mozRequestFullScreen();
        } else if (document.documentElement.webkitRequestFullscreen) {
            document.documentElement.webkitRequestFullscreen(Element.ALLOW_KEYBOARD_INPUT);
        }
    } else {
        if (document.cancelFullScreen) {
            document.cancelFullScreen();
        } else if (document.mozCancelFullScreen) {
            document.mozCancelFullScreen();
        } else if (document.webkitCancelFullScreen) {
            document.webkitCancelFullScreen();
        }
    }
}
window.toggleFullscreen = toggleFullscreen
