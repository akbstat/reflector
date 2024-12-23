pub const TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="en">

<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        .LEVEL1 {
            font-size: 25px;
            font-weight: bold;
        }

        .LEVEL2 {
            font-size: 20px;
            margin-left: 55px;
            font-weight: normal;
        }

        .LEVEL3 {
            font-size: 20px;
            margin-left: 50px;
            text-decoration: none;
            font-weight: normal;
            color: blue;
        }

        .break-page {
            page-break-after: always;
        }

        @media print {
            @page {
                size: A4;
                margin: 40px;
            }
        }
    </style>
</head>

<body>
    <div id="visit"></div>
    <div class="break-page"></div>
    <div id="forms"></div>
</body>

<script type="text/javascript">
    const data = JSON.parse('{{ content }}');
    const visit = data[0];
    const forms = data[1];

    function render(data) {
        let root = document.createElement("div");
        if (!data.children) {
            let link = document.createElement("a");
            link.innerText = data.name;
            link.href = `#${data.id}`;
            link.id = data.id;
            link.className = data.kind;
            root.appendChild(link);
            return root;
        }
        let title = document.createElement("span");
        title.innerText = data.name;
        root.appendChild(title);
        data.children.forEach(child => {
            root.appendChild(render(child));
        });
        root.className = data.kind;
        return root;
    }

    document.getElementById("visit").appendChild(render(visit));
    document.getElementById("forms").appendChild(render(forms));
</script>

</html>
"#;
