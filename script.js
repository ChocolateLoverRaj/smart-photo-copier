const { dialog } = require('electron').remote
const { list: recursiveReaddir } = require('recursive-readdir-async')
const fsExtra = require('fs-extra')
const fs = require('fs')
const { posix: { join, basename, dirname } } = require('path')
const getHash = require('hash-files')

const form = document.getElementById('form')
const existSpan = document.getElementById('form__exist-span')
const srcSpan = document.getElementById('form__src-span')
const destSpan = document.getElementById('form__dest-span')
const progressDiv = document.getElementById('progress')
const gridChecking = document.getElementById('grid__checking')
const gridCopied = document.getElementById('grid__copied')
const gridDuplicate = document.getElementById('grid__duplicate')
const gridRemaining = document.getElementById('grid__remaining')
const textareaChecking = document.getElementById('grid__checking-list')
const textareaCopied = document.getElementById('grid__copied-list')
const textareaDuplicate = document.getElementById('grid__duplicate-list')
const textareaRemaining = document.getElementById('grid__remaining-list')
const stepsList = document.getElementById('steps')

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

let currentStep = 0
const filesChecking = new Set()
const filesCopied = new Set()
const duplicateFiles = new Set()
const remainingFiles = new Set()
const updateProgress = () => {
    for (let i = 0; i < stepsList.children.length; i++) {
        const elem = stepsList.children[i]
        elem.classList.remove('in-progress')
        elem.classList.remove('complete')
        if (i < currentStep) {
            elem.classList.add('complete')
        } else if (i === currentStep) {
            elem.classList.add('in-progress')
        }
    }
    gridChecking.innerText = filesChecking.size
    gridCopied.innerText = filesCopied.size
    gridDuplicate.innerText = duplicateFiles.size
    gridRemaining.innerText = remainingFiles.size
    textareaChecking.innerHTML = [...filesChecking].join('\n')
    textareaCopied.innerHTML = [...filesCopied].join('\n')
    textareaDuplicate.innerHTML = [...duplicateFiles].join('\n')
    textareaRemaining.innerHTML = [...remainingFiles].join('\n')
}

const disable = disable => {
    // Disable copy button
    form.copy.disabled = disable
    // Disable folder inputs
    for (const { button, input } of Object.values(dirs)) {
        button.disabled = disable
        input.disabled = disable
    }
    // Disable compare contents button
    form.compareContents.disabled = disable
}

form.copy.addEventListener('click', async () => {
    // Disable everything
    disable(true)
    // Show progress
    progressDiv.classList.remove('hidden')
    // Reset progress
    currentStep = 0
    filesCopied.clear()
    duplicateFiles.clear()
    remainingFiles.clear()
    updateProgress()
    // Read the exist dir
    const [readExist, readSrc] = await Promise.all([
        recursiveReaddir(form.exist.value),
        recursiveReaddir(form.src.value)
    ])
    currentStep++
    updateProgress()
    // Copy time
    const createDirs = new Map()
    const createDir = path => {
        if (path === '.') {
            return
        }
        if (createDirs.has(path)) {
            return createDirs.get(path)
        }
        const promise = (async () => {
            const parts = path.split('/')
            if (parts.length > 1) {
                await createDir(parts.slice(0, -1).join('/'))
            }
            await fsExtra.ensureDir(join(form.dest.value, path))
        })()
        createDirs.set(path, promise)
        return promise
    }
    const existFiles = new Map(readExist.map(({ fullname, name }) => [name, fullname]))
    console.log(existFiles)
    await Promise.all(readSrc.map(async ({ fullname: path }) => {
        const relativePath = path.slice(form.src.value.length + 1)
        const relativePathFile = basename(relativePath)
        const absolutePath = existFiles.get(relativePathFile)
        let renamedPath
        if (absolutePath) {
            if (!form.compareContents.checked) {
                duplicateFiles.add(relativePath)
                updateProgress()
                return
            }
            filesChecking.add(relativePath)
            updateProgress()
            const [existing, duplicateNamed] = await Promise.all([
                getHash(absolutePath),
                getHash(path)
            ])
            filesChecking.delete(relativePath)
            if (existing.compare(duplicateNamed) === 0) {
                duplicateFiles.add(relativePath)
                updateProgress()
                return
            } else {
                console.log(relativePath, existing, duplicateNamed, existing.compare(duplicateNamed))
            }
            let renamedFile = relativePathFile
            for (let i = 1; existFiles.has(renamedFile); i++) {
                renamedFile = `${relativePathFile} (${i})`
            }
            renamedPath = join(dirname(relativePath), renamedFile)
        }
        // TODO: Chang name of file when copying
        const shownPath = `${relativePath}${renamedPath ? ` -> ${renamedPath}` : ''}`
        remainingFiles.add(shownPath)
        updateProgress()
        await createDir(dirname(relativePath))
        await fs.promises.copyFile(path, join(form.dest.value, renamedPath ?? relativePath))
        remainingFiles.delete(shownPath)
        filesCopied.add(shownPath)
        updateProgress()
    }))
    currentStep++
    updateProgress()
    disable(false)
})
