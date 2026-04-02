# 文件操作函数：解压、模糊选文件并交给编辑器。

function extract
    set -l file $argv[1]
    if test -z "$file"
        echo "extract: 缺少压缩包路径"
        return 1
    end

    if not test -f "$file"
        echo "'$file' 不是有效文件"
        return 1
    end

    if command -q ouch
        ouch decompress $file
        return $status
    end

    switch $file
        case '*.tar.bz2'
            tar xjf $file
        case '*.tar.gz' '*.tgz'
            tar xzf $file
        case '*.tar.xz'
            tar xJf $file
        case '*.bz2'
            bunzip2 $file
        case '*.gz'
            gunzip $file
        case '*.tar'
            tar xf $file
        case '*.zip'
            unzip $file
        case '*.Z'
            uncompress $file
        case '*.7z'
            7z x $file
        case '*.rar'
            unrar x $file
        case '*'
            echo "'$file' 无法识别的压缩格式"
            return 1
    end
end

function fe
    if not command -q fzf
        echo "fe: 缺少 fzf"
        return 1
    end

    if not command -q fd
        echo "fe: 缺少 fd"
        return 1
    end

    set -l preview_cmd 'sed -n "1,200p" {}'
    if command -q bat
        set preview_cmd 'bat --color=always --paging=never --line-range=:200 {}'
    end
    set -l file (fd --type f --hidden --exclude .git | fzf --preview "$preview_cmd")
    if test -n "$file"
        e $file
    end
end
