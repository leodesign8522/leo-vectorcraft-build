# Leo Design Core Bridge v0.10.6 / VectorCraft v0.21.0

���ڣ�2026-11-04���ٷ����ߣ�`11862f61198c8f62c0b22b6c9a331cc5950ac98d`��

## �ֿ�����

1. Core Bridge Ĭ�ϰ棺�ޱ��˵�ģ�顢�˵������/������ء�
2. Leo Design Pie Menu v0.7.4��������ѡ��װ������Ҫ�������� Core��ж�ػָ����� Core��������ǿ������

��ǰ����ӿ�û�л�������������չ�ӿڣ���ѡ��װ���л�ͬ�汾��ǿ EXE������ͨ�� WASM/DLL��
Ĭ�� Core ������ɲ˵� JSON ���ã���ִ���������ò��ᱻ��װ��ɾ�����Ժ���װ��ѡ���ɼ�����ȡ��
����������Ѿ���������ѡ��ж�ز����þ� Core �����³���

## ��������ǿ

- Ctrl ѡ���ڶ�������ê���ƶ�����ɫ�����ߡ�
- AI ԭʼ�༭���ݵ��뼰��Դ���桢PDF/SVG ������ݡ����Ķ����� Ctrl+Enter��
- ��ͼճ����JPEG/PNG/ICO ���������ߴ��������á�
- ������/��ݼ� JSON ���뵼�����ٷ�ԭ����ѡ·���Ρ�
- A ���� Shift �ϵ㱣����ѡê�㣻ҳ��ˮƽ����ֱ��45�㷽�����������ʵ�߽��㣻ԭ�߶��ӳ�������ɫԤ��������ɫ����״̬��

## v0.21 ���ݵ���

- ��ͼճ����ԭ�� CF_BITMAP / GetDIBits ���ղ��� PNG��DIB ����ʧ�ܻ��ˣ�Ctrl+V key-up ���׼�סǰһ֡���μ�״̬��
- ���ߴ��ֶα���ԭ�� Appearance ˲ʱ��������ֶκ�Ĭ��ֵ��
- �ٷ�������������AI ���롢PDF ��ɫ�����ԭ�����Ĳ����ˡ�
- ���ײ��������ڸɾ��ٷ�Դ����Ӧ�ã�craft-ui ���°� Cargo.lock �̶��ύУ�顣

## LC-026 ��������\n\n- Shift �� A �����й̶���ҳ������Լ��ˮƽ����ֱ�� 45�㷽�򣬲��� General > Constrain Angle ��ת��\n- �ӳ���ֻ��ԭ�߶��ƶ��˵���ǰʶ��Ԥ��Ϊ��ɫ���ߣ������ɹ�Ϊ����ɫ���߲���ʾ���е����ǩ��\n\n## �Զ���֤

Ĭ���޲˵����Ӽ�顢������ JSON ��������ѡ�˵������ع飻���ߡ�AI ���롢���塢��ͼ��������ѡ���빤����ָ���ع顣
Windows ��ѡ��װ�����ԣ������ݳ���ܾ�����װ��ȷ�л���ж�غ� Core �ֽ�һ�¡����ñ������������±��������� Shift ˮƽ/��ֱ/45���������ӳ���Ԥ��/����״̬��������ѡ����ר��ع飻PixPin ʵ�������û����ա�
ʵ���û� GUI�������ļ�����װ�����������գ��Զ������ Actions Ϊ׼��

��ʷ ICC �����ռ����ó־û����������ɹ�����ȱ��δ���Ʋ��롣

## Current v0.10.6 verification gate

- Build target: VectorCraft v0.21.0, Core Bridge v0.10.6, optional Pie Menu v0.7.4.
- Required automated checks: Shift-click/Shift-drag selection, 45-degree edge intersections, arbitrary-angle line-extension intersections, release-only Ctrl+V, PNG/DIB/native CF_BITMAP fallback, and paired Pie install/remove.
- A successful build is not PixPin GUI acceptance. Confirm an actual PixPin screenshot paste on Windows before marking LC-025 fully accepted.



## v0.10.6 �� LC-027 / LC-028

- A ����ѡ�е���������߶Σ�Ctrl+C / Ctrl+X ��ȡ��ѡ���Σ����ٸ��������պ�·����
- �������ӹ�ϵ���պ�·�������������߶Ρ�������·���������ֱ��Ͷ�����ʽ�����Ʋ��ı�Դ����
- ����·����ȡ���� live shape ������������༭ʱ�ָ�δ���Ƶıߣ��������Ƽ���ʹ��ԭ��·����
- Shift ˮƽ/��ֱ�ϵ���ƶ�����ѡ���߶�ʱ��Smart Guides �������ɼ��ο��ߣ������� Snap to Point��
- �ӳ���������ɫ����Ԥ��������ɫ���������е㣻�� Shift ��������ʱ������ʾ��һ������ӳ���Ԥ����
- ��֤�ż�������ê�㾫ȷƥ�䣻Rust ���Ρ�����/����/ճ��/SVG/������ο��߷���ع飻���ֳ�����룻��װ�����У�顣
- Windows �� PixPin�������ļ���ʵ�ʲ����ָ�����ʵ�����ա�
