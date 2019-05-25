import {
    default as init,
    makeSimstate,
    setCodeText,
    runAmount,
    step,
    getRegs,
    getErr,
    getMem,
} from './moesi/moesi.js';


/*
 * Constants
 */

const TOTAL_MEM = 0x200;
const CACHELINE_SIZE = 64;
const CACHELINE_WORDS = CACHELINE_SIZE / 4;


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
    function wrap_data(str) {
        return "<div class=\"data\">"
            + str
            + "</div>";
    }
    var html = "<div class=\"cacheline\">";

        html += "<div class=\"label\">" + toHex(addr) + "</div>";
        
        let wordtxt = "<div class=\"memword\">" + toHex(0) + "</div>";
  
        html += wrap_data(
            wrap_data(
                wrap_data(
                    wrap_data(
                        wordtxt.repeat(2)
                    ).repeat(2)
                ).repeat(2)
            ).repeat(2)
        );

    html += "</div>";
    return html;
}

function initMemory() {
    var addr;
    for (addr = 0; addr < TOTAL_MEM; addr += CACHELINE_SIZE) {
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

function updateMem() {
    var mem = new Uint32Array(TOTAL_MEM / 4);
    getMem(simState, mem);

    var cachelines = memory.getElementsByClassName("cacheline");
    for (var line = 0; line < cachelines.length; line++) {

        var words = cachelines[line].getElementsByClassName("memword");

        for (var word = 0; word < words.length; word++) {
            let memoffs = line * CACHELINE_WORDS + word;
            words[word].innerHTML = toHex(mem[memoffs]);
        }

    }
}

function commitCode() {
    setCodeText(simState, textinput.value);
    codeDirty = false;
}



/*
 * Main routine
 */

var textinput = document.getElementById("text-input");
var btnRun = document.getElementById("btn-run");
var btnStep = document.getElementById("btn-step");
var btnReset = document.getElementById("btn-reset");
var fieldInsts = document.getElementById("field-insts");
var breakpoints = document.getElementById("bp-box");

var registers = document.getElementById("registers");
var memory = document.getElementById("memory");

var codeDirty = true;
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
        codeDirty = true;
        var newRows = countLines(event.srcElement.value);
        if (newRows !== codeLines) {
            codeLines = newRows;
            updateBkpts();
        }
    });

    btnRun.addEventListener("click", function(event) {
        commitCode();
        var insts = parseInt(fieldInsts.value);
        if (insts === 0 || isNaN(insts)) insts = 1000;

        let stat = runAmount(simState, insts);
        if (!stat) {
            let err = getErr(simState);
            alert("runToBreak failed! err=" + err);
        }
        updateRegs();
        updateMem();
    });

    btnStep.addEventListener("click", function(event) {
        commitCode();
        let stat = step(simState);
        if (!stat) {
            let err = getErr(simState);
            alert("step failed! err=" + err);
        }
        updateRegs();
        updateMem();
    });

    btnReset.addEventListener("click", function(event) {
        simState = makeSimstate();
        updateRegs();
        updateMem();
        codeDirty = true;
    });
}

run();
