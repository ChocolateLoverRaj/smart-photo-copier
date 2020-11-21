const { dialog } = require('electron').remote
const fs = require('fs')

const form = document.getElementById('form')
const existSpan = document.getElementById('form__exist-span')
const srcSpan = document.getElementById('form__src-span')
const destSpan = document.getElementById('form__dest-span')
const progressDiv = document.getElementById('progress')
const gridCopied = document.getElementById('grid__copied')
const gridDuplicate = document.getElementById('grid__copied')
const gridRemaining = document.getElementById('grid__copied')
const textareaCopied = document.getElementById('grid__copied-list')
const textareaDuplicate = document.getElementById('grid__copied-duplicate')
const textareaRemaining = document.getElementById('grid__copied-remaining')

form.addEventListener('submit', e => {
    e.preventDefault()
})

const dirs = {
    exist: {
        button: form.existButton,
        input: form.exist,
        span: existSpan,
        valid: false
    },
    src: {
        button: form.srcButton,
        input: form.src,
        span: srcSpan,
        valid: false
    },
    dest: {
        button: form.destButton,
        input: form.dest,
        span: destSpan,
        valid: false
    }
}

const updateCopyButton = () => {
    let disabled = false
    for (const { valid } of Object.values(dirs)) {
        if (!valid) {
            disabled = true
            break
        }
    }
    form.copy.disabled = disabled
}

Object.values(dirs).forEach(dir => {
    const checkDirectory = () => {
        const path = dir.input.value.trim()
        dir.valid = false
        if (path) {
            if (fs.existsSync(path)) {
                dir.span.innerText = 'Folder Exists'
                dir.valid = true
            } else {
                dir.span.innerText = 'Folder doesn\'t exist'
            }
        } else {
            dir.span.innerText = 'Not Selected'
        }
        updateCopyButton()
    }
    dir.button.addEventListener('click', () => {
        const path = dialog.showOpenDialogSync({ properties: ['openDirectory'] })
        if (path) {
            dir.input.value = path[0]
            checkDirectory()
        }
    })
    dir.input.addEventListener('input', checkDirectory)
})

// TODO: remove this
form.exist.value = 'C:/users/rajas/source/repos/smart-photo-copier/test/dest'
form.src.value = 'C:/users/rajas/source/repos/smart-photo-copier/test/src/b'
form.dest.value = 'C:/users/rajas/source/repos/smart-photo-copier/test/dest/b'
form.copy.disabled = false

const filesCopied = []
const duplicateFiles = []
const remainingFiles = []
const updateProgress = () => {
    gridCopied.innerText = filesCopied.length
    duplicateFiles.innerText = filesCopied.length
    remainingFiles.innerText = filesCopied.length
    textareaCopied.innerText = filesCopied.join('\n')
    textareaDuplicate.innerText = duplicateFiles.join('\n')
    textareaRemaining.innerText = remainingFiles.join('\n')
}

form.copy.addEventListener('click', () => {
    // Disable copy button
    form.copy.disabled = true
    // Disable everything
    for (const { button, input } of Object.values(dirs)) {
        button.disabled = true
        input.disabled = true
    }
    // Show progress
    progressDiv.classList.remove('hidden')
    // Reset progress
    filesCopied.length = 0
    duplicateFiles.length = 0
    remainingFiles.length = 0
    updateProgress()
})
