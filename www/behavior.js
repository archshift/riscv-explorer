import {
    default as init,
    makeSimstate,
    setCodeText,
    runToBreak,
    getRegs,
    getErr,
} from './moesi/moesi.js';



/*
 * Convenience Functions
 */

function countLines(string) {
    return string.split("\n").length;
}

function toHex(val) {
    var hex = val.toString(16);
    hex = "00000000".substr(0, 8 - hex.length) + hex;
    return hex;
}



/*
 * HTML Initialization Functions
 */

function finishLoading() {
    var elems = document.getElementsByClassName("waitload");
    for (var i = 0; i < elems.length; i++) {
        elems[i].style.visibility = "visible";
    }

    elems = document.getElementsByClassName("whileload");
    for (var i = 0; i < elems.length; i++) {
        elems[i].style.visibility = "hidden";
    }
}

function makeCacheLine(addr) {
    var html = "<div class=\"cacheline\">";

        html += "<div class=\"label\">" + toHex(addr) + "</div>";
        
        var i;
        html += "<div class=\"data\">";
        for (i = 0; i < 16; i++) {
            html += "<div class=\"memword\">" + toHex(0) + "</div>";
        }
        html += "</div>";

    html += "</div>";
    return html;
}

function initMemory() {
    var addr;
    for (addr = 0; addr < 0x200; addr += 64) {
        memory.innerHTML += makeCacheLine(addr);
    }
}

function initRegisters() {
    updateRegs();
}



/*
 * Data Update Functions
 */

function updateBkpts() {
    var row;
    var bps = "";
    for (row = 0; row < codeLines; row++) {
        bps += "<div class=\"bp\"> <div class=\"disabled\"></div> </div>";
    }
    breakpoints.innerHTML = bps;

    var i;
    var bps = breakpoints.getElementsByClassName("bp");
    for (i = 0; i < bps.length; i++) {
        let bpElem = bps[i];
        bpElem.addEventListener("click", toggleBkpt);
    }
}

function toggleBkpt(event) {
    var obj = this.children[0];
    switch (obj.className) {
    case "enabled":
        obj.className = "disabled";
        break;
    case "disabled":
        obj.className = "enabled";
        break;
    }
}

function updateRegs() {
    var regs = new Uint32Array(32);
    getRegs(simState, regs);

    var regElems = registers.getElementsByClassName("regbox");
    for (var i = 0; i < 32; i++) {
        var val = regElems[i].getElementsByClassName("val")[0];
        val.innerHTML = toHex(regs[i]);
    }
}



/*
 * Main routine
 */

var textinput = document.getElementById("text-input");
var btnRun = document.getElementById("btn-run");
var btnStep = document.getElementById("btn-step");
var btnReset = document.getElementById("btn-reset");
var breakpoints = document.getElementById("bp-box");

var registers = document.getElementById("registers");
var memory = document.getElementById("memory");

var codeLines = 0;

var simState = null;
async function run() {
    await init("moesi/moesi_bg.wasm");

    simState = makeSimstate();
    initMemory();
    initRegisters();
    
    codeLines = countLines(textinput.value);
    updateBkpts();
    
    finishLoading();

    textinput.addEventListener("input", function(event) {
        var newRows = countLines(event.srcElement.value);
        if (newRows !== codeLines) {
            codeLines = newRows;
            updateBkpts();
        }
    });

    btnRun.addEventListener("click", function(event) {
        setCodeText(simState, textinput.value);
        let stat = runToBreak(simState);
        if (!stat) {
            let err = getErr(simState);
            alert("runToBreak failed! err=" + err);
        }
        updateRegs();
    });

    btnReset.addEventListener("click", function(event) {
        simState = makeSimstate();
        updateRegs();
    });
}

run();
