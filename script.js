const { dialog } = require('electron').remote
const fs = require('fs')

const form = document.getElementById('form')
const existSpan = document.getElementById('form__exist-span')
const srcSpan = document.getElementById('form__src-span')
const destSpan = document.getElementById('form__dest-span')

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
