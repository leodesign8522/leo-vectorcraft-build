"""Run against an isolated portable build; synthetic text only, no user documents."""
import base64, copy, hashlib, json, socket, subprocess, sys, time
from pathlib import Path

exe = Path(sys.argv[1]).resolve()
assert (exe.parent / 'portable.txt').exists()
startup = subprocess.STARTUPINFO()
startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
startup.wShowWindow = 0
probe = socket.socket(); probe.bind(('127.0.0.1', 0)); port = probe.getsockname()[1]; probe.close()
proc = subprocess.Popen([str(exe), '--new-instance', '--control', str(port),
    '--automation-read-root', str(exe.parent), '--automation-write-root', str(exe.parent)],
    cwd=exe.parent, startupinfo=startup, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
connection = None
try:
    deadline = time.monotonic() + 90
    while time.monotonic() < deadline:
        if proc.poll() is not None: raise RuntimeError('Application exited')
        try:
            connection = socket.create_connection(('127.0.0.1', port), timeout=2); break
        except OSError: time.sleep(.5)
    assert connection is not None
    connection.settimeout(60); stream = connection.makefile('rwb'); serial = 0
    def call(method, params=None):
        global serial
        serial += 1
        stream.write((json.dumps({'id':serial, 'method':method, 'params':params or {}})+'\n').encode()); stream.flush()
        reply = json.loads(stream.readline())
        if not reply.get('ok'): raise RuntimeError(reply)
        return reply.get('result')
    def command(name, **params): return call('engine.execute', {'command':name, 'params':params})
    def node(id): return command('document.node', id=id)
    def ink():
        image = call('ui.render', {'data':True})
        return hashlib.sha256(base64.b64decode(image['pngBase64'])).hexdigest()
    def flag_only(before, after, on):
        expected = copy.deepcopy(before)
        if on: expected['kind']['text']['leoGlyphBounds'] = True
        else: expected['kind']['text'].pop('leoGlyphBounds', None)
        assert expected == after, 'Selection preference must be the only object-data change'
    cases = []
    for index, (text, area, rotated) in enumerate([
        ('a',False,False), ('adfadfadfadfasdf\nasdfasdfasdfa\ndfadfasdf\ndfadfa',True,False),
        ('中文字体\n贴合选框\nAgjp',True,False), ('Agjp',False,True), ('   \n  ',True,False)]):
        command('file.new', width=1200, height=1000)
        p = {'x':180,'y':200,'text':text,'size':100}
        if area: p['area'] = {'width':650,'height':650}
        id = command('text.create', **p)['id']
        command('select.set', ids=[id])
        if rotated: command('object.transform', matrix=[.8,.6,-.6,.8,180,-50])
        command('view.setZoom', zoom=75, center=[500,450])
        call('ui.tool.select', {'tool':'selection'})
        before = node(id); before_pixels = ink()
        if '--capture' in sys.argv and index in [0,1]:
            call('ui.screenshot', {'path':str(exe.parent/f'TEXT_{index}_BEFORE.png'),'data':False})
        command('type.glyphSelectionBounds', on=True)
        enabled = node(id); flag_only(before, enabled, True)
        assert ink() == before_pixels, 'Glyph pixels or line wrapping changed'
        if '--capture' in sys.argv and index in [0,1]:
            call('ui.screenshot', {'path':str(exe.parent/f'TEXT_{index}_TIGHT.png'),'data':False})
        command('edit.undo'); assert node(id) == before
        command('edit.redo'); assert node(id) == enabled
        path = exe.parent/f'glyph-selection-case-{index}.vectorcraft'
        call('app.save', {'path':str(path),'wait':True})
        command('file.close')
        call('app.open', {'path':str(path)})
        assert node(id) == enabled, 'Selection preference did not survive reopen'
        assert ink() == before_pixels
        command('text.setText', id=id, text='Editable text\n仍可编辑')
        assert node(id)['kind']['text']['leoGlyphBounds'] is True
        command('edit.undo'); assert node(id) == enabled
        command('type.glyphSelectionBounds', on=False); flag_only(enabled,node(id),False)
        command('edit.undo'); assert node(id) == enabled
        cases.append({'case':index,'multiline_area':area,'rotated':rotated,'unchanged_render':True,
            'only_selection_preference_changed':True,'save_reopen':True,'editable':True,'undo_redo':True})
    output={'pass':True,'sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'checks':cases}
    (exe.parent/'TEXT_GLYPH_RUNTIME.json').write_text(json.dumps(output,indent=2),encoding='utf-8')
    print(json.dumps(output))
finally:
    if connection is not None:
        try:
            command('file.closeAll')
            for _ in range(8):
                call('ui.dialog.set',{'field':'discard','value':True});call('ui.dialog.confirm')
        except Exception: pass
        try: call('app.quit');proc.wait(timeout=10)
        except Exception: pass
        connection.close()
    if proc.poll() is None: proc.terminate();proc.wait(timeout=10)
