"""Real Windows export command + installed Illustrator edit/save/reopen regression.

Only synthetic documents in an isolated portable build directory are touched.
"""
import hashlib,json,os,subprocess,sys
from pathlib import Path
from native_control import NativeControl

exe=Path(sys.argv[1]).resolve()
root=exe.parent/'AI_EDITABLE_RUNTIME';root.mkdir(exist_ok=True)
with NativeControl(exe) as a:
    a.connection.settimeout(1000)
    def cmd(command,**params):return a.call('engine.execute',{'command':command,'params':params})
    for mode in ['rgb','cmyk']:
        cmd('file.new',width=600,height=500,colorMode=mode)
        cmd('artboard.new',x=650,y=0,width=200,height=150,name='Second')
        layer=a.call('document.inspect')['currentLayer'];cmd('layer.setProps',id=layer,name='Shapes')
        color={'model':'cmyk','c':.1,'m':.2,'y':.3,'k':.4} if mode=='cmyk' else {'model':'rgb','r':.2,'g':.4,'b':.6}
        rect=cmd('shape.rectangle',x=60,y=60,width=180,height=100)['id'];cmd('paint.setFill',color=color,keepModel=True)
        circle=cmd('shape.ellipse',x=300,y=60,width=100,height=100)['id'];cmd('paint.setFill',color=color,keepModel=True)
        cmd('select.set',ids=[rect,circle]);group=cmd('object.group')['id']
        cmd('layer.new',name='Editable Text')
        point=cmd('text.create',x=60,y=230,text='Editable ABC 123',size=32,font='Arial',color=color,keepModel=True)['id']
        area=cmd('text.create',x=60,y=300,text='中文段落文字自动换行测试，保留文字框。\n第二段中文文字。',size=24,font='Microsoft YaHei',color=color,keepModel=True,area={'width':210,'height':150,'fit':'none'})['id']
        cmd('select.set',ids=[point,area])
        before={str(i):a.node(i) for i in [rect,circle,group,point,area]}
        selection=a.call('document.inspect')['selection']
        output=root/(mode+'_EDITABLE.ai')
        result=a.call('ui.menu.invoke',{'command':'file.leoExportEditableAi','params':{'path':str(output),'wait':True}})
        assert output.exists() and output.read_bytes().startswith(b'%PDF-'),result
        assert {str(i):a.node(i) for i in [rect,circle,group,point,area]}==before,'export changed source nodes'
        assert a.call('document.inspect')['selection']==selection,'export changed selection'
        assert not list(root.glob('.leo-ai-export-*')),'owned temporary data was not cleaned'
        a.call('ui.dialog.confirm')
    # A missing font must fail without replacing an existing destination file.
    cmd('file.new',width=300,height=200)
    cmd('text.create',x=30,y=80,text='Missing font',font='LEO_MISSING_FONT_FOR_REGRESSION_0150',size=24)
    destination=root/'MISSING_FONT.ai';destination.write_bytes(b'Existing file must survive')
    failed=False
    try:a.call('ui.menu.invoke',{'command':'file.leoExportEditableAi','params':{'path':str(destination),'wait':True}})
    except RuntimeError:failed=True
    assert destination.read_bytes()==b'Existing file must survive','failed export overwrote destination'
    assert failed,'missing font was silently substituted'
    assert not list(root.glob('.leo-ai-export-*'))

env=os.environ.copy();env['LEO_RUNTIME_ROOT']=str(root)
env['LEO_RUNTIME_SCRIPT']=str(Path(__file__).with_name('ai_export_runtime.jsx').resolve())
code="""$ErrorActionPreference='Stop';[Console]::OutputEncoding=[Text.UTF8Encoding]::new();$ai=$null;try{$ai=New-Object -ComObject Illustrator.Application;$js=[IO.File]::ReadAllText($env:LEO_RUNTIME_SCRIPT,[Text.Encoding]::UTF8);$prefix='var LEO_RUNTIME_ROOT = '+(ConvertTo-Json -InputObject $env:LEO_RUNTIME_ROOT -Compress)+';';$ai.DoJavaScript($prefix+$js)}finally{if($ai){[void][Runtime.InteropServices.Marshal]::ReleaseComObject($ai)}}"""
p=subprocess.run(['powershell.exe','-NoProfile','-NonInteractive','-STA','-Command',code],env=env,capture_output=True,encoding='utf-8',timeout=300)
if p.returncode:raise RuntimeError(p.stderr or p.stdout)
proof={'pass':True,'sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'actual_export_command':True,'source_nodes_and_selection_unchanged':True,'missing_font_preserves_destination':True,'temporary_data_cleaned':True,'illustrator':p.stdout.strip()}
(root/'AI_EDITABLE_RUNTIME.json').write_text(json.dumps(proof,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(proof,ensure_ascii=False))
