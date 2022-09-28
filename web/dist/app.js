const invoke = window.__TAURI__.invoke

invoke('get_local_id').then((id) => document.getElementById("local-id").innerHTML = `<b>ID: </b>${id}`);

$(document).on('click', 'td.add-row', addRow);

$(document).on('click', 'td.remove-row', (e) => {
    if ($(e.currentTarget).parent().parent().children().length > 1) $(e.currentTarget).parent().remove();
});

$(document).on('click', 'td.add-col', addColumn);

$(document).on('click', '.remove-col', (e) => {
    let t = e.target.classList.contains('remove-col') ? $(e.target) : $(e.target).parent();
    let no = t.data('version');
    $(`#settingstable thead th [data-version='${no}']`).parent().remove();
    $(`[data-version='${no}']`).parent().remove();
    $('.add-row').attr('colspan', getNumberOfVersions());
    $('#settingstable tr:not(:last)').each((index, row) => {
        $(row).children().each((index, child) => {
            $(child).children().each((i, e) => {
                $(e).attr('data-version', index + 1);
            })
        })
    });
});

$('#send').on('click', () => {
    let n = getNumberOfVersions();
    let width = 100 / n;
    let input = `send l|${width}`;
    for (let i = 1; i < n; i++) {
        input += `|l|${width}`
    }
    input += "\n";
    $('#settingstable .table-input').each((index, element) => {
        let val = element.value;
        console.log(element, val);
        if (index % n == 0) {
            input += '\n'
        } else {
            input += '|'
        }
        if (!val) val = " ";
        input += val;
    });
    peer = $('#send-peer').val();
    console.log(input);

    invoke('publish_message', {
            message: input,
            peer: peer,
        })
        .then((_) => {})
});

$('#whitelist').on('click', () => {
    peer = $('#whitelist-peer').val();
    if (!peer) return;
    invoke('whitelist', {
        peer: peer,
    }).then((_) => {
        document.getElementById("whitelist-peer").value = "";
    })
});

$('#auth').on('click', () => {
    peer = $('#auth-peer').val();
    if (!peer) return;

    invoke('authorize', {
        peer: peer,
    }).then((_) => {
        document.getElementById("auth-peer").value = "";
    })
});

$('#alias').on('click', () => {
    alias = $('#alias-name').val();
    if (!alias) return;

    document.getElementById("local-id").innerHTML += ` | <b>${alias}</b>`;

    invoke('alias', {
        alias: alias,
    }).then((_) => {
        document.getElementById("alias-name").value = "";
    })
});

function addRow() {
    $('.add-row').parent().before(createRow());
}

function createRow() {
    let row = $('<tr></tr>');
    for (let i = 1; i <= getNumberOfVersions(); i++) {
        row.append('<td><input data-version="' + i + '" class="table-input" type="text"></td>');
    }
    row.append('<td style="text-align: center" class="remove-row"><i class="fas fa-minus-circle"></i></td>');
    return row;
}

function addColumn() {
    let no = getNumberOfVersions() + 1;
    $('.add-col').before(`<th class="align-items-center"><input class="table-input" type="text"><span data-version="${no}" class="remove-col float-right"><i class="fas fa-minus-circle"></i></span></th>`);
    $('.add-row').attr('colspan', no);
    $('.remove-row').before('<td><input data-version="' + no + '"class="table-input" type="text"></td>');
}


/*    "MAKE MY LIFE EASIER" STUFF    */

$('#settingstable').keypress((e) => {
    if (e.keyCode === 13) {
        e.preventDefault();
        if (isSettingsTableComplete()) $('#send').click();
    }
});

function getNumberOfVersions() {
    return $('#settingstable thead tr').children().length - 1;
}


$('#fill-timetable').click(() => {
    addRow();
    addRow();
    addRow();
    addRow();
    addRow();
    addColumn();
    $('#settingstable thead input:eq(1)').val('T9/112');
    $('#settingstable tbody tr:eq(0) input:first').val('08 - 10');
    $('#settingstable tbody tr:eq(1) input:first').val('10 - 12');
    $('#settingstable tbody tr:eq(2) input:first').val('12 - 14');
    $('#settingstable tbody tr:eq(3) input:first').val('14 - 16');
    $('#settingstable tbody tr:eq(4) input:first').val('16 - 18');
});