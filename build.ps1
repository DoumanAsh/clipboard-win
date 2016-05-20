Write-Host "------------------------------"
Write-Host "Clipboard-win build script"
Write-Host "------------------------------"

if ( $args.length -lt 1 ) {
    Write-Host "Usage: build command [options]"
    Write-Host ""
    Write-Host "Commands:"
    Write-Host "    test            Run cargo test without threading."
    Write-Host "    build           Run cargo build."
    Write-Host "    doc             Generate docs."
    Write-Host ""
    Write-Host "Options:"
    Write-Host "    --update_pages  For doc command copy documents to branch gh-pages."
    Write-Host ""
    exit
}

Set-Location $PSScriptRoot #just in case set the location to script(Clipboard-win directory)

switch ($args[0])
{
    "test" {
        echo "let's save some clipboard!" | Clip
        $env:RUST_TEST_THREADS=1
        cargo test
    }
    "build" {
        cargo build
    }
    "doc"   {
        $master_hash = git log -1 --format="%s(%h %cd)" --date=short
        #remove old documentation just in case
        Remove-Item -Recurse -ErrorAction SilentlyContinue -Force target\doc\
        cargo doc --no-deps -p windows-error -p clipboard-win
        if ($args[1] -eq "--update-pages") {
            git checkout gh-pages -q
            Remove-Item -Recurse -ErrorAction SilentlyContinue -Force doc\
            Copy-item -Recurse target\doc\ doc\
            git diff --quiet HEAD
            if ($LASTEXITCODE -eq 1) {
                #commit change in docs
                git add doc/
                git commit -m "Auto-update" -m "Commit: $master_hash"
                git push origin HEAD
            }
            else {
                echo ""
                echo "Documents are up-to-date"
            }
            git checkout master -q
        }
    }
    "bot" {
        if ( -Not $env:APPVEYOR) {
            echo "Bot is supposed to run in AppVeyor. Exit..."
            return
        }
        elseif ($env:APPVEYOR_PULL_REQUEST_TITLE) {
            echo "Skip pull request"
            return
        }
        elseif ( $env:TARGET -ne "x86_64-pc-windows-gnu") {
            return
        }

        git config --global credential.helper store
        Add-Content "$env:USERPROFILE\.git-credentials" "https://$($env:git_token):x-oauth-basic@github.com\n"
        git config --global user.name "AppVeyor bot"
        git config --global user.email "douman@gmx.se"
        git config remote.origin.url "https://$($env:git_token)@github.com/DoumanAsh/clipboard-win.git"
        echo ""
        echo "Build is done"

        $crates_io_ver=cargo search clipboard-win
        $crates_io_ver = $crates_io_ver -match "\d{1,3}.\d{1,3}.\d{1,3}"
        $crates_io_ver = $crates_io_ver.split()[1]
        $crates_io_ver = $crates_io_ver.substring(1, $crates_io_ver.indexof(")")-1).split('.')

        $crate = select-string Cargo.toml -Pattern "\d{1,3}.\d{1,3}.\d{1,3}"
        $crate = $crate[0].tostring().split('=')[1].substring(2)
        $crate = $crate.substring(0, $crate.indexof('"')).split('.')

        $crates_io_ver = [System.Tuple]::Create($crates_io_ver[0], $crates_io_ver[1], $crates_io_ver[2])
        $crate = [System.Tuple]::Create($crate[0], $crate[1], $crate[2])
        if ( $crate[0] -gt $crates_io_ver[0]) {
            cargo login $env:api
            cargo publish
            if ($LASTEXITCODE -eq 1) {
                echo ""
                echo "Unable to publish :("
            }
            else {
                .\build.ps1 doc --update-pages
            }
        }
        else {
            echo ""
            echo "Crate is up-to-date on crates.io"
        }
    }
    default { echo (">>>{0}: Incorrect command" -f $args[0]) }
}
