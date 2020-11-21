const { dialog } = require('electron').remote
const fs = require('fs')

const form = document.getElementById('form')
const existSpan = document.getElementById('form__exist-span')

form.addEventListener('submit', e => {
    e.preventDefault()
})

form.existButton.addEventListener('click', e => {
    const path = dialog.showOpenDialogSync({ properties: ['openDirectory'] })
    if (path) {
        form.exist.value = path[0]
        updateExistSpan()
    }
})


const updateExistSpan = () => {
    const path = form.exist.value.trim()
    existSpan.innerText = path
        ? fs.existsSync(path) ? 'Valid Folder' : 'Folder doesn\'t exist'
        : 'Not Selected'
}
form.exist.addEventListener('input', updateExistSpan)